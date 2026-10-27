//! Framework-neutral chart maths: scales, nice ticks and mark geometry.
//! Adapters turn the output into markup; nothing here knows about the DOM.

pub mod annotate;
pub mod bar;
pub mod boxplot;
pub mod brush;
pub mod candle;
pub mod histogram;
pub mod timeline;
pub mod donut;
pub mod field;
pub mod geo;
pub mod radar;
pub mod rank;
pub mod smith;
pub mod hierarchy;
pub mod network;
pub mod sankey;
pub mod sunburst;
pub mod treemap;
pub mod heatmap;
pub mod line;
pub mod nav;
pub mod radial;
pub mod scatter;
pub mod scale;
pub mod sparkline;
pub mod stacked;

/// Which way bars run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Bars grow upwards, categories along the bottom.
    #[default]
    Vertical,
    /// Bars grow rightwards, categories down the left. Suits long labels.
    Horizontal,
}

impl Orientation {
    /// The `data-orientation` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            Orientation::Vertical => "vertical",
            Orientation::Horizontal => "horizontal",
        }
    }
}

/// One labelled value.
#[derive(Debug, Clone, PartialEq)]
pub struct Datum {
    /// Category name, shown on the axis or legend.
    pub label: String,
    /// The value. Non-finite values are treated as zero.
    pub value: f64,
}

impl Datum {
    /// A labelled value.
    #[must_use]
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self { label: label.into(), value }
    }
}

/// Plain number formatting for labels: whole numbers without decimals, others to one place.
#[must_use]
pub fn format_value(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

/// Keep at most `max_slots` categories: the largest `max_slots - 1` stay in their
/// original order and the rest fold into one "Other" entry at the end.
/// Categorical palettes have six hues; a seventh series is never a new colour.
#[must_use]
pub fn fold_other(data: &[Datum], max_slots: usize) -> Vec<Datum> {
    if data.len() <= max_slots || max_slots < 2 {
        return data.to_vec();
    }
    let mut by_size: Vec<usize> = (0..data.len()).collect();
    by_size.sort_by(|&a, &b| data[b].value.total_cmp(&data[a].value));
    let keep: std::collections::HashSet<usize> = by_size.into_iter().take(max_slots - 1).collect();
    let mut out: Vec<Datum> = data.iter().enumerate().filter(|(i, _)| keep.contains(i)).map(|(_, d)| d.clone()).collect();
    let rest: f64 = data.iter().enumerate().filter(|(i, _)| !keep.contains(i)).map(|(_, d)| d.value).sum();
    out.push(Datum::new("Other", rest));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_the_smallest_into_other_keeping_order() {
        let data: Vec<Datum> = [("a", 5.0), ("b", 1.0), ("c", 9.0), ("d", 2.0)].iter().map(|(l, v)| Datum::new(*l, *v)).collect();
        let folded = fold_other(&data, 3);
        let labels: Vec<&str> = folded.iter().map(|d| d.label.as_str()).collect();
        assert_eq!(labels, ["a", "c", "Other"]);
        assert!((folded[2].value - 3.0).abs() < 1e-9);
    }

    #[test]
    fn formats_whole_and_fractional_values() {
        assert_eq!(format_value(42.0), "42");
        assert_eq!(format_value(4.25), "4.2");
    }
}
