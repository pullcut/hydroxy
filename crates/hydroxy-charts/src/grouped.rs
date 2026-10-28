//! Grouped bars: several series side by side within each category, from a zero baseline.

use crate::annotate::Annotation;
use crate::line::Series;
use crate::plot::{ValueAxis, band_at};
use crate::scale::ScaleKind;

/// Share of each category band the group of bars fills; the rest is the gap between groups.
const GROUP: f64 = 0.76;

/// One bar. Category-axis fractions run from the start of the axis; value fractions from the baseline side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupedBar {
    /// Series index (the bar's colour).
    pub series: usize,
    /// Category index.
    pub category: usize,
    /// Centre along the category axis, 0..=1.
    pub centre: f64,
    /// Thickness along the category axis, 0..=1.
    pub width: f64,
    /// The baseline end, 0..=1.
    pub from: f64,
    /// The value end, 0..=1.
    pub to: f64,
}

/// Everything a grouped bar chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupedLayout {
    /// Every present value, category by category.
    pub bars: Vec<GroupedBar>,
    /// The value axis, always including zero.
    pub axis: ValueAxis,
}

#[allow(clippy::cast_precision_loss, reason = "bar counts are small")]
/// Lay out `series` over `categories` categories. Missing or non-finite values leave an empty slot.
#[must_use]
pub fn layout(series: &[Series], categories: usize, annotations: &[Annotation]) -> GroupedLayout {
    let values = series.iter().flat_map(|s| s.values.iter().take(categories).copied());
    let axis = ValueAxis::new(ScaleKind::Linear, values, annotations, true);
    let (n, m) = (categories.max(1) as f64, series.len().max(1) as f64);
    let slot = GROUP / m;
    let base = axis.base();
    let bars = (0..categories)
        .flat_map(|c| {
            let axis = &axis;
            series.iter().enumerate().filter_map(move |(s, data)| {
                let v = data.values.get(c).copied().filter(|v| v.is_finite())?;
                let centre = (c as f64 + (1.0 - GROUP) / 2.0 + (s as f64 + 0.5) * slot) / n;
                Some(GroupedBar { series: s, category: c, centre, width: slot * 0.86 / n, from: base, to: axis.at(v) })
            })
        })
        .collect();
    GroupedLayout { bars, axis }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "bar counts are small"
)]
/// The (series, category) slot at position `p` along the category axis, for pointer hover.
#[must_use]
pub fn pick(p: f64, categories: usize, series: usize) -> Option<(usize, usize)> {
    let c = band_at(p, categories)?;
    let local = p.clamp(0.0, 1.0) * categories as f64 - c as f64;
    let s = ((local - (1.0 - GROUP) / 2.0) / (GROUP / series.max(1) as f64)).max(0.0) as usize;
    Some((s.min(series.saturating_sub(1)), c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_sit_side_by_side_from_zero() {
        let s = [Series::new("A", [10.0, -5.0]), Series::new("B", [20.0, f64::NAN])];
        let l = layout(&s, 2, &[]);
        assert_eq!(l.bars.len(), 3, "the NaN slot is empty");
        assert!(l.bars[0].centre < l.bars[1].centre && l.bars[1].centre < 0.5);
        let neg = l.bars[2];
        assert!(neg.to < neg.from, "negative bars hang below the baseline");
        assert!(l.axis.scale.domain.0 < 0.0);
    }

    #[test]
    fn pick_finds_the_bar_under_the_pointer() {
        assert_eq!(pick(0.2, 2, 2), Some((0, 0)));
        assert_eq!(pick(0.3, 2, 2), Some((1, 0)));
        assert_eq!(pick(0.99, 2, 2), Some((1, 1)));
        assert_eq!(pick(0.5, 0, 2), None);
    }
}
