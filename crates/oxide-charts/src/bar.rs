//! Vertical bar chart layout. Heights are fractions of the plot area, so adapters can
//! lay bars out with plain CSS and stay responsive without measuring anything.

use crate::scale::{Linear, ticks};

/// One bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    /// Height as a fraction of the plot area, 0..=1.
    pub fraction: f64,
    /// Drawn in the accent colour (the latest or selected value).
    pub highlighted: bool,
}

/// A horizontal gridline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gridline {
    /// The tick value.
    pub value: f64,
    /// Distance from the baseline as a fraction of the plot area, 0..=1.
    pub fraction: f64,
}

/// Everything a bar chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct BarLayout {
    /// One per value, in order.
    pub bars: Vec<Bar>,
    /// Gridlines from the baseline up, including the top of the nice domain.
    pub gridlines: Vec<Gridline>,
    /// Top of the value axis (a nice round number at or above the largest value).
    pub max: f64,
}

/// Which bar gets the accent colour.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Highlight {
    /// The last bar: the latest value in a time series.
    #[default]
    Last,
    /// The bar at this index.
    Index(usize),
    /// No accent; every bar uses the series colour.
    Off,
}

impl Highlight {
    /// The highlighted index for `len` bars, if any.
    #[must_use]
    pub fn resolve(self, len: usize) -> Option<usize> {
        match self {
            Highlight::Last => len.checked_sub(1),
            Highlight::Index(i) if i < len => Some(i),
            Highlight::Index(_) | Highlight::Off => None,
        }
    }
}

/// Lay out bars for `values`, starting at zero.
/// `highlight` picks one bar for the accent colour, usually the latest.
// zero-based and non-negative only; diverging bars need a centred baseline.
#[must_use]
pub fn layout(values: &[f64], highlight: Option<usize>, tick_count: usize) -> BarLayout {
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    let largest = clean.iter().copied().fold(0.0, f64::max);
    let tick_values = ticks(0.0, if largest > 0.0 { largest } else { 1.0 }, tick_count);
    let max = tick_values.last().copied().unwrap_or(1.0);
    let scale = Linear::new((0.0, max), (0.0, 1.0));
    BarLayout {
        bars: clean
            .iter()
            .enumerate()
            .map(|(i, &v)| Bar { fraction: scale.map(v).clamp(0.0, 1.0), highlighted: highlight == Some(i) })
            .collect(),
        gridlines: tick_values.iter().map(|&value| Gridline { value, fraction: scale.map(value) }).collect(),
        max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_scale_against_a_nice_maximum() {
        let l = layout(&[12.0, 30.0, 87.0], Some(2), 5);
        assert!((l.max - 100.0).abs() < 1e-9);
        assert!((l.bars[2].fraction - 0.87).abs() < 1e-9);
        assert!(l.bars[2].highlighted && !l.bars[0].highlighted);
        assert_eq!(l.gridlines.first().map(|g| g.fraction), Some(0.0));
        assert_eq!(l.gridlines.last().map(|g| g.fraction), Some(1.0));
    }

    #[test]
    fn negatives_and_nan_sit_on_the_baseline() {
        let l = layout(&[-5.0, f64::NAN, 4.0], None, 5);
        assert_eq!(l.bars[0].fraction, 0.0);
        assert_eq!(l.bars[1].fraction, 0.0);
    }

    #[test]
    fn all_zero_values_still_get_an_axis() {
        let l = layout(&[0.0, 0.0], None, 5);
        assert!(l.max > 0.0);
    }
}
