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
    layout_with_extent(values, highlight, tick_count, None)
}

/// [`layout`] with the axis raised to cover the top of `extent` (such as
/// [`crate::annotate::y_extent`]), so a target line above every bar stays visible.
#[must_use]
pub fn layout_with_extent(values: &[f64], highlight: Option<usize>, tick_count: usize, extent: Option<(f64, f64)>) -> BarLayout {
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    let top = extent.map_or(0.0, |e| e.1).max(0.0);
    let largest = clean.iter().copied().fold(top, f64::max);
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

/// Turn a placed annotation (from [`crate::annotate::place`] in vertical terms: x across the
/// categories, y up the values) into horizontal-bar terms: values run left to right and
/// categories top to bottom.
#[must_use]
pub fn transpose(p: crate::annotate::Placed) -> crate::annotate::Placed {
    use crate::annotate::{Placed, Shape};
    let shape = match p.shape {
        Shape::HLine => Shape::VLine,
        Shape::VLine => Shape::HLine,
        Shape::HBand => Shape::VBand,
        Shape::VBand => Shape::HBand,
        Shape::Point => Shape::Point,
    };
    Placed { shape, x0: p.y0, x1: p.y1, y0: 1.0 - p.x1, y1: 1.0 - p.x0, label: p.label }
}

#[allow(clippy::cast_precision_loss, reason = "category counts are small")]
/// The centre of category `i` of `len`, 0..=1, for placing annotations over bars.
#[must_use]
pub fn category_centre(i: f64, len: usize) -> f64 {
    (i + 0.5) / len.max(1) as f64
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
    fn extents_raise_the_axis_and_transpose_swaps_axes() {
        assert!(layout_with_extent(&[10.0], None, 5, Some((0.0, 140.0))).max >= 140.0);
        let placed = crate::annotate::place(&[crate::annotate::Annotation::y_line(50.0, "Target")], |i| category_centre(i, 4), |v| v / 100.0);
        let t = transpose(placed[0].clone());
        assert_eq!(t.shape, crate::annotate::Shape::VLine);
        assert!((t.x0 - 0.5).abs() < 1e-12);
        assert!((category_centre(1.0, 4) - 0.375).abs() < 1e-12);
    }

    #[test]
    fn all_zero_values_still_get_an_axis() {
        let l = layout(&[0.0, 0.0], None, 5);
        assert!(l.max > 0.0);
    }
}
