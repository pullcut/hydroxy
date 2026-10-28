//! Radar chart: several series over N axes radiating from one centre.
#![allow(clippy::cast_precision_loss, reason = "axis counts are small")]

use std::f64::consts::{FRAC_PI_2, TAU};
use std::fmt::Write as _;

use crate::scale::nice_domain;

/// One axis spoke.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarAxis {
    /// Outer end of the spoke.
    pub end: (f64, f64),
    /// Where the label goes.
    pub label: (f64, f64),
    /// SVG `text-anchor` for the label.
    pub anchor: &'static str,
}

/// One series polygon.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarShape {
    /// A vertex per axis.
    pub points: Vec<(f64, f64)>,
    /// Closed SVG `d`.
    pub path: String,
}

/// Everything a radar chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Radar {
    /// viewBox width.
    pub width: f64,
    /// viewBox height.
    pub height: f64,
    /// Centre of the web.
    pub centre: (f64, f64),
    /// Spokes, clockwise from the top.
    pub axes: Vec<RadarAxis>,
    /// Ring polygons as `(value, d)`, innermost first.
    pub rings: Vec<(f64, String)>,
    /// One polygon per series.
    pub shapes: Vec<RadarShape>,
    /// Value at the outer ring (the axes start at zero).
    pub max: f64,
}

fn polygon(points: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        let _ = write!(d, "{}{x:.2} {y:.2}", if i == 0 { "M" } else { "L" });
    }
    if !points.is_empty() {
        d.push('Z');
    }
    d
}

/// Lay out `series` (one value per axis each) on `axes` spokes with `rings` guide rings.
/// `max` fixes the outer value; otherwise it is the data maximum rounded up to a nice number.
#[must_use]
pub fn layout(axes: usize, series: &[Vec<f64>], rings: usize, max: Option<f64>) -> Radar {
    let (width, height, radius) = (440.0, 380.0, 130.0);
    let centre = (width / 2.0, height / 2.0 + 4.0);
    let data_max = series.iter().flatten().copied().filter(|v| v.is_finite()).fold(0.0, f64::max);
    let max = max.filter(|m| *m > 0.0).unwrap_or_else(|| nice_domain(0.0, data_max.max(f64::EPSILON), rings + 1).1);
    let angle = |i: usize| -FRAC_PI_2 + TAU * i as f64 / axes.max(1) as f64;
    let at = |i: usize, r: f64| (centre.0 + r * angle(i).cos(), centre.1 + r * angle(i).sin());
    let spokes = (0..axes)
        .map(|i| {
            let c = angle(i).cos();
            RadarAxis {
                end: at(i, radius),
                label: at(i, radius + 14.0),
                anchor: if c > 0.2 {
                    "start"
                } else if c < -0.2 {
                    "end"
                } else {
                    "middle"
                },
            }
        })
        .collect();
    let rings = (1..=rings)
        .map(|k| {
            let f = k as f64 / rings as f64;
            (max * f, polygon(&(0..axes).map(|i| at(i, radius * f)).collect::<Vec<_>>()))
        })
        .collect();
    let shapes = series
        .iter()
        .map(|s| {
            let points: Vec<(f64, f64)> = (0..axes)
                .map(|i| {
                    at(i, radius * (s.get(i).copied().filter(|v| v.is_finite()).unwrap_or(0.0) / max).clamp(0.0, 1.0))
                })
                .collect();
            RadarShape { path: polygon(&points), points }
        })
        .collect();
    Radar { width, height, centre, axes: spokes, rings, shapes, max }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_axis_points_up_and_values_scale_to_max() {
        let r = layout(4, &[vec![10.0, 10.0, 0.0, 5.0]], 4, Some(10.0));
        assert!(r.axes[0].end.1 < r.centre.1 && (r.axes[0].end.0 - r.centre.0).abs() < 1e-9);
        assert!((r.shapes[0].points[0].1 - r.axes[0].end.1).abs() < 1e-9);
        assert!((r.shapes[0].points[2].1 - r.centre.1).abs() < 1e-9);
        assert_eq!(r.rings.len(), 4);
        assert!((r.rings[3].0 - 10.0).abs() < 1e-9);
    }

    #[test]
    fn nice_max_covers_the_data() {
        assert!(layout(3, &[vec![72.0, 40.0, 91.0]], 4, None).max >= 91.0);
    }
}
