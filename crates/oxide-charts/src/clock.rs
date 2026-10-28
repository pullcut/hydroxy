//! 24-hour clock layout: a value per hour drawn as a radial bar out from an inner ring,
//! midnight at 12 o'clock and running clockwise, with an optional second series beside
//! each bar. Paths sit in a 0..100 viewBox centred on (50, 50).

use crate::polar;
use crate::scale::{nice_domain, ticks};
use std::f64::consts::TAU;

/// Outer radius of the face in viewBox units; hour labels sit just outside it.
pub const R: f64 = 50.0;
/// Where bars start, viewBox units.
pub const INNER: f64 = 16.0;
/// Where a bar at the top of the scale ends.
pub const OUTER: f64 = 44.0;

/// One bar: a radial line from the inner ring outwards.
#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    /// Slot (hour) index.
    pub slot: usize,
    /// Series index, 0 or 1.
    pub series: usize,
    /// SVG `d` from the inner ring to the bar's end, for a stroked path.
    pub path: String,
    /// The value, `None` for a gap.
    pub value: Option<f64>,
}

/// A reference ring at a tick value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    /// The value it marks.
    pub value: f64,
    /// Radius in viewBox units.
    pub radius: f64,
}

/// Everything a clock chart needs.
#[derive(Debug, Clone, PartialEq)]
pub struct ClockLayout {
    /// Bars in slot order, each slot's series side by side.
    pub bars: Vec<Bar>,
    /// Value rings from the inside out.
    pub rings: Vec<Ring>,
    /// One wedge per slot from the centre to the face, for hit testing.
    pub hits: Vec<String>,
    /// Bar thickness in viewBox units.
    pub width: f64,
    /// Slots around the face.
    pub slots: usize,
}

/// Radius for `value` on a scale topping out at `top`.
fn reach(value: f64, top: f64) -> f64 {
    INNER + (value / top).clamp(0.0, 1.0) * (OUTER - INNER)
}

/// Angle through the middle of slot `i` of `n`, radians clockwise from 12.
#[allow(clippy::cast_precision_loss, reason = "slot counts are tiny")]
#[must_use]
pub fn mid(i: usize, n: usize) -> f64 {
    (i as f64 + 0.5) / n.max(1) as f64 * TAU
}

/// Lay out one or two series (the first sets the slot count, usually 24) on a shared
/// zero-based scale.
#[allow(clippy::cast_precision_loss, reason = "slot counts are tiny")]
#[must_use]
pub fn layout(series: &[&[f64]]) -> ClockLayout {
    let series = &series[..series.len().min(2)];
    let slots = series.first().map_or(0, |s| s.len());
    let peak = series.iter().flat_map(|s| s.iter()).filter(|v| v.is_finite()).fold(0.0_f64, |m, &v| m.max(v));
    let (_, top) = nice_domain(0.0, if peak > 0.0 { peak } else { 1.0 }, 4);
    let slot = TAU / slots.max(1) as f64;
    // Bars fill about half their slot at the inner ring, split between the series.
    let width = (INNER * slot * 0.6 / series.len().max(1) as f64).min(3.6);
    let bars = (0..slots)
        .flat_map(|i| series.iter().enumerate().map(move |(s, values)| (i, s, values.get(i).copied())))
        .map(|(i, s, v)| {
            let value = v.filter(|v| v.is_finite());
            // Two series sit either side of the slot's centre line, a bar's width apart.
            let offset = if series.len() > 1 { (s as f64 - 0.5) * width * 1.15 } else { 0.0 };
            let angle = mid(i, slots);
            let (nx, ny) = (angle.cos(), angle.sin());
            let at = |r: f64| (polar::R + r * angle.sin() + offset * nx, polar::R - r * angle.cos() + offset * ny);
            let ((x0, y0), (x1, y1)) = (at(INNER), at(reach(value.unwrap_or(0.0).max(0.0), top).max(INNER + 0.6)));
            Bar { slot: i, series: s, path: format!("M{x0:.3} {y0:.3}L{x1:.3} {y1:.3}"), value }
        })
        .collect();
    let rings = ticks(0.0, top, 4).into_iter().filter(|&t| t > 0.0).map(|value| Ring { value, radius: reach(value, top) }).collect();
    let hits = (0..slots).map(|i| polar::wedge_path(R, i as f64 * slot, (i + 1) as f64 * slot)).collect();
    ClockLayout { bars, rings, hits, width, slots }
}

/// Where a tooltip for slot `i` anchors, as fractions of the face from its top-left.
#[must_use]
pub fn anchor(i: usize, n: usize) -> (f64, f64) {
    polar::at((OUTER * 0.8) / R, mid(i, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_reach_out_by_value() {
        let values: Vec<f64> = (0..24).map(f64::from).collect();
        let l = layout(&[&values]);
        assert_eq!((l.bars.len(), l.slots, l.hits.len()), (24, 24, 24));
        assert_eq!(l.rings.last().map(|r| r.radius), Some(OUTER));
        // The first bar points just clockwise of 12 o'clock and starts on the inner ring.
        let (x, y) = polar::at(INNER / R, mid(0, 24));
        assert!(l.bars[0].path.starts_with(&format!("M{:.3} {:.3}", x * 100.0, y * 100.0)));
    }

    #[test]
    fn two_series_sit_side_by_side() {
        let a = [5.0; 24];
        let b = [10.0; 24];
        let l = layout(&[&a, &b, &a]);
        assert_eq!(l.bars.len(), 48, "a third series is ignored");
        assert_eq!((l.bars[0].series, l.bars[1].series), (0, 1));
        assert_ne!(l.bars[0].path[..12], l.bars[1].path[..12]);
    }

    #[test]
    fn empty_and_gappy_input() {
        assert_eq!(layout(&[]).bars.len(), 0);
        let l = layout(&[&[f64::NAN, 2.0]]);
        assert_eq!(l.bars[0].value, None);
        let (x, y) = anchor(0, 4);
        assert!(x > 0.5 && y < 0.5);
    }
}
