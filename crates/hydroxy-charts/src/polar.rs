//! Polar area (Nightingale rose): every category gets the same angle and its radius
//! encodes the value, on a zero-based scale. Paths sit in a 0..100 viewBox centred
//! on (50, 50); angles start at 12 o'clock and run clockwise.

use crate::scale::{nice_domain, ticks};
use std::f64::consts::{PI, TAU};

/// Outer radius of the plot in viewBox units.
pub const R: f64 = 50.0;

/// One wedge.
#[derive(Debug, Clone, PartialEq)]
pub struct Wedge {
    /// SVG `d` for the filled wedge. Empty for zero or negative values.
    pub path: String,
    /// Radius as a fraction of the outer ring, 0..=1.
    pub radius: f64,
    /// Angle through the middle of the wedge, radians clockwise from 12 o'clock.
    pub mid: f64,
}

/// A reference ring at a tick value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    /// The value it marks.
    pub value: f64,
    /// Radius as a fraction of the outer ring.
    pub radius: f64,
}

/// Everything a polar area chart needs.
#[derive(Debug, Clone, PartialEq)]
pub struct PolarLayout {
    /// One per value.
    pub wedges: Vec<Wedge>,
    /// Reference rings from the inside out; the last is the outer edge.
    pub rings: Vec<Ring>,
}

fn point(r: f64, angle: f64) -> (f64, f64) {
    (R + r * angle.sin(), R - r * angle.cos())
}

/// SVG `d` for a filled wedge of radius `r` from `start` to `end` radians, centred in the viewBox.
#[must_use]
pub fn wedge_path(r: f64, start: f64, end: f64) -> String {
    if r <= 0.0 || end <= start {
        return String::new();
    }
    if end - start >= TAU - 1e-9 {
        let (x, y) = point(r, 0.0);
        let (bx, by) = point(r, PI);
        return format!("M{x:.3} {y:.3}A{r:.3} {r:.3} 0 1 1 {bx:.3} {by:.3}A{r:.3} {r:.3} 0 1 1 {x:.3} {y:.3}Z");
    }
    let (x0, y0) = point(r, start);
    let (x1, y1) = point(r, end);
    let large = i32::from(end - start > PI);
    format!("M{R:.3} {R:.3}L{x0:.3} {y0:.3}A{r:.3} {r:.3} 0 {large} 1 {x1:.3} {y1:.3}Z")
}

/// Point at `radius` (fraction of the outer ring) and `angle`, as fractions of the box from its top-left.
#[must_use]
pub fn at(radius: f64, angle: f64) -> (f64, f64) {
    let (x, y) = point(radius * R, angle);
    (x / (2.0 * R), y / (2.0 * R))
}

#[allow(clippy::cast_precision_loss, reason = "category counts are small")]
/// Lay out `values` with `gap` radians between neighbouring wedges.
#[must_use]
pub fn layout(values: &[f64], gap: f64) -> PolarLayout {
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    let max = clean.iter().copied().fold(0.0, f64::max);
    let (_, top) = nice_domain(0.0, if max > 0.0 { max } else { 1.0 }, 4);
    let n = clean.len();
    let span = if n > 0 { TAU / n as f64 } else { TAU };
    let half_gap = if n > 1 { gap / 2.0 } else { 0.0 };
    let wedges = clean
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let start = i as f64 * span;
            let radius = v / top;
            Wedge { path: wedge_path(radius * R, start + half_gap, start + span - half_gap), radius, mid: start + span / 2.0 }
        })
        .collect();
    let rings = ticks(0.0, top, 4).into_iter().filter(|&t| t > 0.0).map(|value| Ring { value, radius: value / top }).collect();
    PolarLayout { wedges, rings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_angles_and_radius_by_value() {
        let l = layout(&[10.0, 20.0, 40.0, 0.0], 0.0);
        assert_eq!(l.wedges.len(), 4);
        assert!((l.wedges[2].radius - 1.0).abs() < 1e-9);
        assert!((l.wedges[1].radius - 0.5).abs() < 1e-9);
        assert_eq!(l.wedges[3].path, "");
        // Quarter turns: the second wedge's middle points at 4:30.
        assert!((l.wedges[1].mid - 3.0 * PI / 4.0).abs() < 1e-9);
        assert_eq!(l.rings.last().map(|r| r.radius), Some(1.0));
    }

    #[test]
    fn first_wedge_starts_at_twelve_and_one_value_is_a_circle() {
        let l = layout(&[10.0, 10.0], 0.0);
        assert!(l.wedges[0].path.starts_with("M50.000 50.000L50.000 0.000"));
        let one = layout(&[10.0], 0.0);
        assert_eq!(one.wedges[0].path.matches('A').count(), 2);
    }

    #[test]
    fn points_map_to_box_fractions() {
        let (x, y) = at(1.0, PI / 2.0);
        assert!((x - 1.0).abs() < 1e-9 && (y - 0.5).abs() < 1e-9);
    }
}
