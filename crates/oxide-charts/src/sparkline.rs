//! Sparklines: a tiny line with no axes, in a fixed pixel box. The value axis spans
//! the data's own range (not zero), since a sparkline shows shape, not size.

use crate::line::{Curve, area, path, runs};
use crate::scale::Linear;

/// Everything a sparkline needs to draw, in the `width`×`height` box it was laid out for.
#[derive(Debug, Clone, PartialEq)]
pub struct SparklineLayout {
    /// SVG `d` for the line.
    pub line: String,
    /// SVG `d` for the area under the line, down to the bottom of the box.
    pub area: String,
    /// Centre of the last point, if the last value is present.
    pub last: Option<(f64, f64)>,
    /// Smallest finite value.
    pub min: Option<f64>,
    /// Largest finite value.
    pub max: Option<f64>,
    /// The last value, if finite.
    pub latest: Option<f64>,
}

#[allow(clippy::cast_precision_loss, reason = "point counts are small")]
/// Lay out `values` in a `width`×`height` box, inset by `pad` on every side so strokes
/// and the last-point dot are not clipped. Non-finite values leave gaps.
#[must_use]
pub fn layout(values: &[f64], width: f64, height: f64, pad: f64, curve: Curve) -> SparklineLayout {
    let finite = || values.iter().copied().filter(|v| v.is_finite());
    let min = finite().reduce(f64::min);
    let max = finite().reduce(f64::max);
    let (lo, hi) = (min.unwrap_or(0.0), max.unwrap_or(0.0));
    let bottom = height - pad;
    // A flat line sits in the middle.
    let y = if hi > lo { Linear::new((lo, hi), (bottom, pad)) } else { Linear::new((lo - 1.0, lo + 1.0), (bottom, pad)) };
    let n = values.len();
    let x = |i: usize| if n > 1 { pad + (width - 2.0 * pad) * i as f64 / (n - 1) as f64 } else { width / 2.0 };
    let points: Vec<Option<(f64, f64)>> =
        values.iter().enumerate().map(|(i, &v)| v.is_finite().then(|| (x(i), y.map(v)))).collect();
    let runs = runs(&points);
    SparklineLayout {
        line: runs.iter().map(|r| path(r, curve)).collect(),
        area: runs.iter().map(|r| area(r, bottom, curve)).collect(),
        last: points.last().copied().flatten(),
        min,
        max,
        latest: values.last().copied().filter(|v| v.is_finite()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_the_box_inside_the_padding() {
        let s = layout(&[1.0, 3.0, 2.0], 100.0, 20.0, 2.0, Curve::Linear);
        assert_eq!(s.line, "M2.000 18.000L50.000 2.000L98.000 10.000");
        assert_eq!(s.last, Some((98.0, 10.0)));
        assert_eq!((s.min, s.max, s.latest), (Some(1.0), Some(3.0), Some(2.0)));
        assert!(s.area.ends_with("L98.000 18.000L2.000 18.000Z"));
    }

    #[test]
    fn flat_and_empty_data() {
        let s = layout(&[5.0, 5.0], 10.0, 10.0, 1.0, Curve::Linear);
        assert_eq!(s.line, "M1.000 5.000L9.000 5.000");
        let e = layout(&[], 10.0, 10.0, 1.0, Curve::Linear);
        assert_eq!((e.line.as_str(), e.last, e.min), ("", None, None));
    }

    #[test]
    fn a_missing_last_value_has_no_dot() {
        let s = layout(&[1.0, 2.0, f64::NAN], 10.0, 10.0, 1.0, Curve::Linear);
        assert_eq!((s.last, s.latest), (None, None));
    }
}
