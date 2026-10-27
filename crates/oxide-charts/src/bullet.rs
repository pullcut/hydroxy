//! Bullet rows: a measure bar against a target tick, over qualitative range bands.
//! Each row has its own zero-based scale. Positions are fractions of the track.

use crate::bar::Gridline;
use crate::scale::{nice_domain, ticks};

/// One measure.
#[derive(Debug, Clone, PartialEq)]
pub struct BulletRow {
    /// What it measures, such as "Revenue".
    pub label: String,
    /// The current value.
    pub value: f64,
    /// The goal, drawn as a tick across the bar.
    pub target: Option<f64>,
    /// Upper edges of the qualitative bands, such as poor, fair and good. Any order.
    pub ranges: Vec<f64>,
}

impl BulletRow {
    /// A row with a value only; add a target and ranges with the builders.
    #[must_use]
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self { label: label.into(), value, target: None, ranges: Vec::new() }
    }

    /// Set the target.
    #[must_use]
    pub fn target(mut self, target: f64) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the band edges.
    #[must_use]
    pub fn ranges(mut self, ranges: impl Into<Vec<f64>>) -> Self {
        self.ranges = ranges.into();
        self
    }
}

/// A row in track fractions.
#[derive(Debug, Clone, PartialEq)]
pub struct RowLayout {
    /// Length of the measure bar, 0..=1.
    pub value: f64,
    /// Position of the target tick.
    pub target: Option<f64>,
    /// Bands as (start, end) fractions, from the lowest up.
    pub bands: Vec<(f64, f64)>,
    /// The band edges sorted ascending, matching `bands`.
    pub edges: Vec<f64>,
    /// Axis ticks along the track.
    pub ticks: Vec<Gridline>,
    /// Value at the right end of the track.
    pub max: f64,
}

fn clean(v: f64) -> f64 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

/// Lay out one row.
#[must_use]
pub fn layout(row: &BulletRow) -> RowLayout {
    let mut edges: Vec<f64> = row.ranges.iter().copied().map(clean).collect();
    edges.sort_by(f64::total_cmp);
    let top = edges.iter().copied().chain([clean(row.value), row.target.map_or(0.0, clean)]).fold(0.0, f64::max);
    let (_, max) = nice_domain(0.0, if top > 0.0 { top } else { 1.0 }, 4);
    let f = |v: f64| (clean(v) / max).min(1.0);
    let mut lo = 0.0;
    let bands = edges
        .iter()
        .map(|&e| {
            let band = (lo, f(e));
            lo = f(e);
            band
        })
        .collect();
    RowLayout {
        value: f(row.value),
        target: row.target.map(f),
        bands,
        edges,
        ticks: ticks(0.0, max, 4).into_iter().map(|value| Gridline { value, fraction: value / max }).collect(),
        max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_scales_to_the_largest_of_value_target_and_ranges() {
        let l = layout(&BulletRow::new("Revenue", 270.0).target(250.0).ranges([300.0, 150.0, 225.0]));
        assert_eq!(l.max, 300.0);
        assert_eq!(l.edges, [150.0, 225.0, 300.0]);
        assert_eq!(l.bands, [(0.0, 0.5), (0.5, 0.75), (0.75, 1.0)]);
        assert!((l.value - 0.9).abs() < 1e-9);
        assert_eq!(l.target.map(|t| (t * 1000.0).round()), Some(833.0));
        assert_eq!(l.ticks.first().map(|t| t.value), Some(0.0));
    }

    #[test]
    fn empty_and_negative_rows_stay_on_the_track() {
        let l = layout(&BulletRow::new("Empty", -5.0));
        assert_eq!((l.value, l.target, l.bands.len()), (0.0, None, 0));
        assert!(l.max > 0.0);
    }
}
