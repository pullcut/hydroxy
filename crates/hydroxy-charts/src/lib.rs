//! Framework-neutral chart maths: scales, nice ticks and mark geometry.
//! Adapters turn the output into markup; nothing here knows about the DOM.

pub mod alluvial;
pub mod annotate;
pub mod arc_diagram;
pub mod bar;
pub mod bland_altman;
pub mod boxplot;
pub mod brush;
pub mod bullet;
pub mod bundle;
pub mod calendar;
pub mod candle;
pub mod cartogram;
pub mod chart_table;
pub mod chord;
pub mod clock;
pub mod cohort;
pub mod compare;
pub mod composed;
pub mod connected;
pub mod csv;
pub mod cycle;
pub mod dag;
pub mod delaunay;
pub mod density;
pub mod depth;
pub mod diverging;
pub mod donut;
pub mod ecdf;
pub mod event;
pub mod field;
pub mod flow_map;
pub mod forest;
pub mod funnel;
pub mod geo;
pub mod geom;
pub mod grouped;
pub mod heatmap;
pub mod hexbin;
pub mod hierarchy;
pub mod histogram;
pub mod hive;
pub mod horizon;
pub mod interval;
pub mod line;
pub mod marey;
pub mod marimekko;
pub mod matrix;
pub mod metric;
pub mod nav;
pub mod nested_donut;
pub mod network;
pub mod ohlc;
pub mod pack;
pub mod parallel;
pub mod partition;
pub mod pictogram;
pub mod pie;
pub mod plot;
pub mod polar;
pub mod probe;
pub mod qq;
pub mod quiver;
pub mod race;
pub mod radar;
pub mod radial;
pub mod raincloud;
pub mod rank;
pub mod raster;
pub mod renko;
pub mod sankey;
pub mod scale;
pub mod scatter;
pub mod small_multiples;
pub mod smith;
pub mod span;
pub mod sparkline;
pub mod spectrogram;
pub mod spiral;
pub mod split;
pub mod stacked;
pub mod stacked_area;
pub mod step;
pub mod sunburst;
pub mod survival;
pub mod swarm;
pub mod ternary;
pub mod tidy;
pub mod timeline;
pub mod treemap;
pub mod upset;
pub mod venn;
pub mod voronoi;
pub mod waffle;
pub mod waterfall;
pub mod wordcloud;
pub mod zoom;

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
    if (value - value.round()).abs() < 1e-9 { format!("{value:.0}") } else { format!("{value:.1}") }
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
    let mut out: Vec<Datum> =
        data.iter().enumerate().filter(|(i, _)| keep.contains(i)).map(|(_, d)| d.clone()).collect();
    let rest: f64 = data.iter().enumerate().filter(|(i, _)| !keep.contains(i)).map(|(_, d)| d.value).sum();
    out.push(Datum::new("Other", rest));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_the_smallest_into_other_keeping_order() {
        let data: Vec<Datum> =
            [("a", 5.0), ("b", 1.0), ("c", 9.0), ("d", 2.0)].iter().map(|(l, v)| Datum::new(*l, *v)).collect();
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
