//! Spiral layout: a time series wound onto an Archimedean spiral, one turn per period
//! (a year of days, a week of hours), so the same point in each period lines up along a
//! spoke. Each value is a short arc of the spiral shaded by value. Paths sit in a
//! 0..100 viewBox centred on (50, 50); angles start at 12 o'clock and run clockwise.

use std::f64::consts::TAU;
use std::fmt::Write as _;

use crate::scale;

/// Outer radius of the plot in viewBox units.
pub const R: f64 = 50.0;
/// Radius where the first turn starts, as a share of [`R`].
const HOLE: f64 = 0.22;
/// Points per arc, so the curve stays smooth however few values there are.
const STEPS: usize = 4;

/// One value's arc.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// SVG `d` along the spiral, for a stroked path.
    pub path: String,
    /// The value, `None` for a gap.
    pub value: Option<f64>,
    /// Position on the colour ramp, 0..=1.
    pub level: f64,
    /// When its stroke starts drawing, 0..=1 of the whole draw.
    pub start: f64,
    /// How long it takes, a share of the whole draw.
    pub span: f64,
}

/// Everything a spiral chart needs.
#[derive(Debug, Clone, PartialEq)]
pub struct SpiralLayout {
    /// One per value, in order.
    pub segments: Vec<Segment>,
    /// Distance between turns, viewBox units; strokes are a little thinner.
    pub spacing: f64,
    /// Radius of the first turn's start.
    pub inner: f64,
    /// Turns drawn (whole or partial).
    pub turns: usize,
    /// Smallest finite value.
    pub min: f64,
    /// Largest finite value.
    pub max: f64,
}

/// Point at radius `r` and `angle` (radians clockwise from 12) in the viewBox.
#[must_use]
pub fn point(r: f64, angle: f64) -> (f64, f64) {
    crate::geom::polar(R, r, angle)
}

fn radius(l: &SpiralLayout, t: f64) -> f64 {
    l.inner + l.spacing * t
}

/// When the pen reaches share `f` of the values, as a share of the draw. The pen eases
/// out (fast from the centre, settling on the last turn), so this is the inverse of
/// `1 - (1 - t)²`.
fn reach(f: f64) -> f64 {
    1.0 - (1.0 - f.clamp(0.0, 1.0)).sqrt()
}

/// Wind `values` onto a spiral with `period` values per turn.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "value counts are small"
)]
#[allow(clippy::many_single_char_names, reason = "polar geometry reads best in x, y, r, t")]
#[must_use]
pub fn layout(values: &[f64], period: f64) -> SpiralLayout {
    let period = if period.is_finite() && period >= 1.0 { period } else { 1.0 };
    let n = values.len();
    let turns = ((n as f64 / period).ceil() as usize).max(1);
    let inner = R * HOLE;
    // The last turn's outer edge sits half a spacing inside R.
    let spacing = (R - inner) / (turns as f64 + 0.5);
    let finite = |v: f64| Some(v).filter(|v| v.is_finite());
    let (min, max) = scale::extent(values.iter().copied()).unwrap_or((0.0, 0.0));
    let mut l = SpiralLayout { segments: Vec::with_capacity(n), spacing, inner, turns, min, max };
    for (i, &v) in values.iter().enumerate() {
        let mut path = String::new();
        for s in 0..=STEPS {
            let t = (i as f64 + s as f64 / STEPS as f64) / period;
            let (x, y) = point(radius(&l, t), t * TAU);
            let _ = write!(path, "{}{x:.3} {y:.3}", if s == 0 { 'M' } else { 'L' });
        }
        let value = finite(v);
        let level = value.map_or(0.0, |v| if max > min { (v - min) / (max - min) } else { 1.0 });
        let (a, b) = (reach(i as f64 / n as f64), reach((i + 1) as f64 / n as f64));
        l.segments.push(Segment { path, value, level, start: a, span: b - a });
    }
    l
}

/// The value index under a point given as fractions (`x`, `y`) of the square plot from its
/// top-left, or `None` when it falls between turns or outside the data.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "turn maths on small counts"
)]
#[allow(clippy::many_single_char_names, reason = "polar geometry reads best in x, y, r, t")]
#[must_use]
pub fn index_at(l: &SpiralLayout, period: f64, x: f64, y: f64) -> Option<usize> {
    let (dx, dy) = (x * 2.0 * R - R, y * 2.0 * R - R);
    let r = dx.hypot(dy);
    let turn = dx.atan2(-dy).rem_euclid(TAU) / TAU;
    let k = ((r - l.inner) / l.spacing - turn).round();
    if k < 0.0 || (radius(l, k + turn) - r).abs() > l.spacing / 2.0 {
        return None;
    }
    let i = ((k + turn) * period).floor();
    (i >= 0.0 && (i as usize) < l.segments.len()).then_some(i as usize)
}

/// Where value `i` sits, as fractions of the plot from its top-left (for a tooltip).
#[allow(clippy::cast_precision_loss, reason = "value counts are small")]
#[allow(clippy::many_single_char_names, reason = "polar geometry reads best in x, y, r, t")]
#[must_use]
pub fn anchor(l: &SpiralLayout, period: f64, i: usize) -> (f64, f64) {
    let t = (i as f64 + 0.5) / period;
    let (x, y) = point(radius(l, t), t * TAU);
    (x / (2.0 * R), y / (2.0 * R))
}

/// Spoke `j` of `count` (a dial label such as a month): SVG `d` for a faint line across
/// every turn, and where its label sits outside the spiral, as fractions of the plot.
#[allow(clippy::cast_precision_loss, reason = "spoke counts are tiny")]
#[must_use]
pub fn spoke(l: &SpiralLayout, j: usize, count: usize) -> (String, (f64, f64)) {
    let angle = j as f64 / count.max(1) as f64 * TAU;
    let (x0, y0) = point(l.inner - l.spacing / 2.0, angle);
    let (x1, y1) = point(R, angle);
    let (lx, ly) = point(R + 5.0, angle);
    (format!("M{x0:.3} {y0:.3}L{x1:.3} {y1:.3}"), (lx / (2.0 * R), ly / (2.0 * R)))
}

/// Where turn `k`'s label sits: on the spiral at 12 o'clock, as fractions of the plot.
#[allow(clippy::cast_precision_loss, reason = "turn counts are tiny")]
#[must_use]
pub fn turn_anchor(l: &SpiralLayout, k: usize) -> (f64, f64) {
    let (x, y) = point(radius(l, k as f64), 0.0);
    (x / (2.0 * R), y / (2.0 * R))
}

/// Stroke for a value at `level` (0..=1): `--chart-1` mixed into the sunken surface, the
/// same ramp as the heatmap.
#[must_use]
pub fn shade(level: f64) -> String {
    format!("color-mix(in oklab, var(--chart-1) {:.1}%, var(--surface-sunken))", 8.0 + level.clamp(0.0, 1.0) * 92.0)
}

/// Which side of its anchor a label sits on, for `data-side`.
#[must_use]
pub fn side(x: f64) -> &'static str {
    if x < 0.45 {
        "left"
    } else if x > 0.55 {
        "right"
    } else {
        "center"
    }
}

/// Keyboard movement: Left/Right step one value along the spiral, Up/Down a whole turn
/// (out and in), Home/End jump, Escape clears.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "period is a small positive count")]
#[must_use]
pub fn key(key: &str, current: Option<usize>, len: usize, period: f64) -> Option<Option<usize>> {
    let turn = (period.round() as usize).max(1);
    let last = len.checked_sub(1)?;
    match (key, current) {
        ("ArrowDown", Some(i)) => Some(Some(i.saturating_sub(turn))),
        ("ArrowUp", Some(i)) => Some(Some((i + turn).min(last))),
        _ => crate::nav::list_key(key, current, len),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_turn_per_period() {
        let l = layout(&[1.0; 30], 10.0);
        assert_eq!(l.turns, 3);
        assert!(l.segments[0].path.starts_with("M50.000 39.000"), "{}", l.segments[0].path);
        // Value 10 starts a turn further out, straight up again.
        let (_, y) = point(l.inner + l.spacing, 0.0);
        assert!(l.segments[10].path.starts_with(&format!("M50.000 {y:.3}")));
        assert!(radius(&l, 3.0) + l.spacing / 2.0 <= R + 1e-9);
    }

    #[test]
    fn draw_timing_covers_the_whole_sweep() {
        let l = layout(&[1.0, 2.0, 3.0, f64::NAN], 4.0);
        assert!(l.segments[0].start.abs() < 1e-9);
        let end = l.segments[3].start + l.segments[3].span;
        assert!((end - 1.0).abs() < 1e-9);
        assert!(l.segments[0].span < l.segments[3].span, "the pen slows down");
        assert_eq!(l.segments[3].value, None);
        assert!((l.segments[1].level - 0.5).abs() < 1e-9);
    }

    #[test]
    fn keys_step_values_and_turns() {
        assert_eq!(key("ArrowUp", Some(2), 30, 10.0), Some(Some(12)));
        assert_eq!(key("ArrowDown", Some(2), 30, 10.0), Some(Some(0)));
        assert_eq!(key("ArrowRight", Some(2), 30, 10.0), Some(Some(3)));
        assert_eq!(key("ArrowUp", Some(25), 30, 10.0), Some(Some(29)));
        let l = layout(&[1.0; 20], 10.0);
        let (_, (x, y)) = spoke(&l, 1, 4);
        assert!(x > 1.0 && (y - 0.5).abs() < 1e-9, "3 o'clock, outside the plot");
        assert!((turn_anchor(&l, 0).0 - 0.5).abs() < 1e-9);
        assert_eq!(side(0.2), "left");
    }

    #[test]
    fn pointer_finds_the_value_under_it() {
        let period = 12.0;
        let l = layout(&[1.0; 36], period);
        for i in [0, 5, 13, 30, 35] {
            let (x, y) = anchor(&l, period, i);
            assert_eq!(index_at(&l, period, x, y), Some(i));
        }
        assert_eq!(index_at(&l, period, 0.5, 0.5), None, "the hole in the middle");
        assert_eq!(index_at(&l, period, 0.0, 0.0), None, "outside the last turn");
    }
}
