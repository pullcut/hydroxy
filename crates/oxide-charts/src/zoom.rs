//! Zoom and pan, ported from hecto's chart-core zoom. A [`View`] is the visible window
//! of a plot as fractions (0..=1) of its full extent on each axis, so it works for any
//! scale: charts map data to full-extent fractions, then [`View::project_x`] /
//! [`View::project_y`] place them in the visible plot.

use crate::scale::{ScaleKind, ValueScale};

/// An interval `(start, end)` with `start <= end`.
pub type Span = (f64, f64);

/// Keep `span` inside `bounds` and at least `min_span` wide, preserving its width where
/// possible by sliding it back in rather than cutting it.
#[must_use]
pub fn clamp(span: Span, bounds: Span, min_span: f64) -> Span {
    let (lo, hi) = if span.0 <= span.1 { span } else { (span.1, span.0) };
    let (min, max) = bounds;
    let bound = max - min;
    if bound <= 0.0 {
        return bounds;
    }
    let width = (hi - lo).max(min_span);
    if width >= bound {
        return bounds;
    }
    let centre = f64::midpoint(lo, hi);
    let (mut start, mut end) = (centre - width / 2.0, centre + width / 2.0);
    if start < min {
        end += min - start;
        start = min;
    }
    if end > max {
        start -= end - max;
        end = max;
    }
    (start, end)
}

/// Zoom `span` by `factor` (above 1 zooms in) keeping `anchor` where it is on screen.
#[must_use]
pub fn zoom_around(span: Span, anchor: f64, factor: f64, bounds: Span, min_span: f64) -> Span {
    if !factor.is_finite() || factor <= 0.0 {
        return clamp(span, bounds, min_span);
    }
    let width = (span.1 - span.0).max(f64::EPSILON);
    let anchor = anchor.clamp(span.0, span.1);
    let next = width / factor;
    let start = anchor - next * (anchor - span.0) / width;
    clamp((start, start + next), bounds, min_span)
}

/// Move `span` by `delta`, stopping at the bounds.
#[must_use]
pub fn pan(span: Span, delta: f64, bounds: Span) -> Span {
    let width = span.1 - span.0;
    let start = (span.0 + delta).clamp(bounds.0, (bounds.1 - width).max(bounds.0));
    (start, start + width)
}

/// The data value at full-extent fraction `f` of `scale` (the inverse of [`ValueScale::map`]).
#[must_use]
pub fn invert(scale: ValueScale, fraction: f64) -> f64 {
    let (lo, hi) = scale.domain;
    match scale.kind {
        ScaleKind::Linear => lo + fraction * (hi - lo),
        ScaleKind::Log => 10f64.powf(lo.log10() + fraction * (hi.log10() - lo.log10())),
        ScaleKind::Symlog => {
            let forward = |v: f64| v.signum() * v.abs().ln_1p();
            let t = forward(lo) + fraction * (forward(hi) - forward(lo));
            t.signum() * t.abs().exp_m1()
        }
    }
}

/// Which axes a chart zooms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Axes {
    /// Only the x axis (line, area and candlestick charts).
    #[default]
    X,
    /// Both axes (scatter charts).
    XY,
}

/// The visible part of a plot, as fractions of its full extent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// Visible x window.
    pub x: Span,
    /// Visible y window.
    pub y: Span,
}

const UNIT: Span = (0.0, 1.0);

impl Default for View {
    fn default() -> Self {
        Self::FULL
    }
}

impl View {
    /// Nothing zoomed.
    pub const FULL: View = View { x: UNIT, y: UNIT };

    /// Whether the whole plot is visible.
    #[must_use]
    pub fn is_full(self) -> bool {
        self.x.0 <= 1e-9 && self.x.1 >= 1.0 - 1e-9 && self.y.0 <= 1e-9 && self.y.1 >= 1.0 - 1e-9
    }

    /// Where full-extent x fraction `f` sits in the visible plot (outside 0..=1 when hidden).
    #[must_use]
    pub fn project_x(self, f: f64) -> f64 {
        (f - self.x.0) / (self.x.1 - self.x.0).max(f64::EPSILON)
    }

    /// Where full-extent y fraction `f` sits in the visible plot.
    #[must_use]
    pub fn project_y(self, f: f64) -> f64 {
        (f - self.y.0) / (self.y.1 - self.y.0).max(f64::EPSILON)
    }

    /// The full-extent fractions under visible-plot point `(fx, fy)`.
    #[must_use]
    pub fn unproject(self, fx: f64, fy: f64) -> (f64, f64) {
        (self.x.0 + fx * (self.x.1 - self.x.0), self.y.0 + fy * (self.y.1 - self.y.0))
    }

    /// Zoom by `factor` (above 1 zooms in) around visible-plot point `at`.
    /// No window gets narrower than `min_span`.
    #[must_use]
    pub fn zoom(self, at: (f64, f64), factor: f64, axes: Axes, min_span: f64) -> View {
        let (ax, ay) = self.unproject(at.0, at.1);
        View {
            x: zoom_around(self.x, ax, factor, UNIT, min_span),
            y: if axes == Axes::XY { zoom_around(self.y, ay, factor, UNIT, min_span) } else { self.y },
        }
    }

    /// Drag the content by `(dx, dy)` visible-plot fractions: the data follows the pointer.
    #[must_use]
    pub fn pan_by(self, dx: f64, dy: f64, axes: Axes) -> View {
        View {
            x: pan(self.x, -dx * (self.x.1 - self.x.0), UNIT),
            y: if axes == Axes::XY { pan(self.y, -dy * (self.y.1 - self.y.0), UNIT) } else { self.y },
        }
    }

    /// Keyboard zoom: `+` and `-` zoom around the centre, Shift+arrows pan by a tenth,
    /// `0` resets. `None` when the key is not handled or nothing would change.
    #[must_use]
    pub fn key(self, key: &str, shift: bool, axes: Axes, min_span: f64) -> Option<View> {
        let next = match (key, shift) {
            ("+" | "=", _) => self.zoom((0.5, 0.5), 1.25, axes, min_span),
            ("-" | "_", _) => self.zoom((0.5, 0.5), 0.8, axes, min_span),
            ("0", _) => View::FULL,
            ("ArrowLeft", true) => self.pan_by(0.1, 0.0, axes),
            ("ArrowRight", true) => self.pan_by(-0.1, 0.0, axes),
            ("ArrowUp", true) if axes == Axes::XY => self.pan_by(0.0, -0.1, axes),
            ("ArrowDown", true) if axes == Axes::XY => self.pan_by(0.0, 0.1, axes),
            _ => return None,
        };
        (next != self).then_some(next)
    }
}

/// The zoom factor for a wheel event's vertical delta in pixels: scrolling up zooms in.
#[must_use]
pub fn wheel_factor(delta_y: f64) -> f64 {
    (-delta_y.clamp(-200.0, 200.0) * 0.002).exp()
}

/// A two-finger pinch from `before` to `after` (two points each, in visible-plot
/// fractions): the zoom factor and the anchor (the midpoint after the move).
#[must_use]
pub fn pinch(before: [(f64, f64); 2], after: [(f64, f64); 2]) -> (f64, (f64, f64)) {
    let dist = |p: [(f64, f64); 2]| (p[0].0 - p[1].0).hypot(p[0].1 - p[1].1);
    let factor = if dist(before) > 1e-6 { dist(after) / dist(before) } else { 1.0 };
    (factor, (f64::midpoint(after[0].0, after[1].0), f64::midpoint(after[0].1, after[1].1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Span, b: Span) -> bool {
        (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
    }

    #[test]
    fn clamp_slides_back_inside_and_keeps_width() {
        assert!(close(clamp((-0.2, 0.3), UNIT, 0.0), (0.0, 0.5)));
        assert!(close(clamp((0.8, 1.3), UNIT, 0.0), (0.5, 1.0)));
        assert!(close(clamp((0.4, 0.41), UNIT, 0.1), (0.355, 0.455)));
        assert!(close(clamp((-1.0, 2.0), UNIT, 0.0), UNIT));
    }

    #[test]
    fn zoom_keeps_the_anchor_still() {
        let z = zoom_around(UNIT, 0.25, 2.0, UNIT, 0.01);
        assert!(close(z, (0.125, 0.625)));
        // The anchor sits a quarter of the way across before and after.
        assert!(((0.25 - z.0) / (z.1 - z.0) - 0.25).abs() < 1e-9);
        assert!(close(zoom_around(z, 0.3, 0.1, UNIT, 0.01), UNIT), "zooming far out stops at the bounds");
        assert!((zoom_around(UNIT, 0.5, 1000.0, UNIT, 0.1).1 - zoom_around(UNIT, 0.5, 1000.0, UNIT, 0.1).0 - 0.1).abs() < 1e-9);
    }

    #[test]
    fn pan_stops_at_the_ends() {
        assert!(close(pan((0.2, 0.4), 0.1, UNIT), (0.3, 0.5)));
        assert!(close(pan((0.2, 0.4), 5.0, UNIT), (0.8, 1.0)));
        assert!(close(pan((0.2, 0.4), -5.0, UNIT), (0.0, 0.2)));
    }

    #[test]
    fn views_project_zoom_pan_and_take_keys() {
        let v = View::FULL.zoom((0.5, 0.5), 2.0, Axes::X, 0.01);
        assert!(close(v.x, (0.25, 0.75)) && v.y == UNIT, "x-only charts never zoom y");
        assert!((v.project_x(0.5) - 0.5).abs() < 1e-9 && (v.project_x(0.25)).abs() < 1e-9);
        let panned = v.pan_by(0.5, 0.0, Axes::X);
        assert!(close(panned.x, (0.0, 0.5)), "dragging right reveals earlier data");
        let xy = View::FULL.zoom((0.0, 0.0), 2.0, Axes::XY, 0.01);
        assert!(close(xy.x, (0.0, 0.5)) && close(xy.y, (0.0, 0.5)));
        assert_eq!(View::FULL.key("-", false, Axes::X, 0.01), None, "already fully out");
        assert!(View::FULL.key("+", false, Axes::X, 0.01).is_some_and(|v| !v.is_full()));
        assert_eq!(v.key("0", false, Axes::X, 0.01), Some(View::FULL));
        assert!(v.key("ArrowRight", true, Axes::X, 0.01).is_some_and(|n| n.x.0 > v.x.0));
        assert_eq!(v.key("ArrowRight", false, Axes::X, 0.01), None, "plain arrows stay with the crosshair");
        assert_eq!(v.key("ArrowUp", true, Axes::X, 0.01), None);
    }

    #[test]
    fn invert_undoes_map_for_every_scale() {
        for kind in [ScaleKind::Linear, ScaleKind::Log, ScaleKind::Symlog] {
            let s = ValueScale::new(kind, -50.0_f64.max(if kind == ScaleKind::Log { 1.0 } else { -50.0 }), 5000.0, 5);
            for v in [2.0, 30.0, 700.0] {
                assert!((invert(s, s.map(v)) - v).abs() < 1e-6 * v.abs().max(1.0), "{kind:?} {v}");
            }
        }
    }

    #[test]
    fn wheel_and_pinch_factors() {
        assert!(wheel_factor(-100.0) > 1.0 && wheel_factor(100.0) < 1.0);
        let (f, at) = pinch([(0.4, 0.5), (0.6, 0.5)], [(0.3, 0.5), (0.7, 0.5)]);
        assert!((f - 2.0).abs() < 1e-9 && close(at, (0.5, 0.5)));
    }
}
