//! A brush: a window `[start, end]` within 0..=1 that zooms a plot. Pure state with
//! pointer drags and keyboard edge adjustment; adapters supply fractions of the strip.

/// A selected window of the x axis, as fractions of its full extent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brush {
    /// Left edge.
    pub start: f64,
    /// Right edge.
    pub end: f64,
}

/// Which part of the brush a drag or key moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    /// The left edge.
    Start,
    /// The right edge.
    End,
    /// The whole window, keeping its width.
    Pan,
}

/// A drag in progress.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drag {
    /// What is being moved.
    pub grab: Grab,
    /// Where the pointer went down.
    pub anchor: f64,
    /// The brush when the drag began.
    pub origin: Brush,
}

impl Brush {
    /// The whole axis: nothing zoomed.
    pub const FULL: Brush = Brush { start: 0.0, end: 1.0 };

    /// Width of the window.
    #[must_use]
    pub fn span(self) -> f64 {
        self.end - self.start
    }

    /// Whether the window covers the whole axis.
    #[must_use]
    pub fn is_full(self) -> bool {
        self.start <= 0.0 && self.end >= 1.0
    }

    /// Start a drag at fraction `f`. Within `tol` of an edge grabs that edge (the nearer one);
    /// inside the window pans it; outside, the window first recentres on `f` and then pans.
    /// Returns the brush to show and the drag to continue with [`Drag::apply`].
    #[must_use]
    pub fn begin(self, f: f64, tol: f64) -> (Brush, Drag) {
        let f = f.clamp(0.0, 1.0);
        let (ds, de) = ((f - self.start).abs(), (f - self.end).abs());
        let grab = if ds.min(de) <= tol && self.span() > tol {
            if ds <= de { Grab::Start } else { Grab::End }
        } else {
            Grab::Pan
        };
        let brush = if grab == Grab::Pan && (f < self.start || f > self.end) {
            let start = (f - self.span() / 2.0).clamp(0.0, 1.0 - self.span());
            Brush { start, end: start + self.span() }
        } else {
            self
        };
        (brush, Drag { grab, anchor: f, origin: brush })
    }

    /// Move one edge to `f`, keeping at least `min_span` between the edges.
    #[must_use]
    pub fn with_edge(self, grab: Grab, f: f64, min_span: f64) -> Brush {
        let min_span = min_span.clamp(0.0, 1.0);
        match grab {
            Grab::Start => Brush { start: f.clamp(0.0, (self.end - min_span).max(0.0)), end: self.end },
            Grab::End => Brush { start: self.start, end: f.clamp((self.start + min_span).min(1.0), 1.0) },
            Grab::Pan => {
                let start = f.clamp(0.0, 1.0 - self.span());
                Brush { start, end: start + self.span() }
            }
        }
    }

    /// Keyboard adjustment of one edge: Left/Right move it by `step` (ten steps with
    /// Shift), Home and End send it to the lowest or highest allowed position.
    /// `None` when `key` is not handled or nothing would change.
    #[must_use]
    pub fn key(self, grab: Grab, key: &str, shift: bool, step: f64, min_span: f64) -> Option<Brush> {
        let step = if shift { step * 10.0 } else { step };
        let at = if grab == Grab::End { self.end } else { self.start };
        let target = match key {
            "ArrowLeft" | "ArrowDown" => at - step,
            "ArrowRight" | "ArrowUp" => at + step,
            "Home" => 0.0,
            "End" => 1.0,
            _ => return None,
        };
        let next = self.with_edge(grab, target, min_span);
        (next != self).then_some(next)
    }

    /// The `lo..hi` index range of `xs` (ascending positions in 0..=1) inside the window.
    /// Always at least two points when there are two or more, so a line can be drawn.
    #[must_use]
    pub fn range(self, xs: &[f64]) -> std::ops::Range<usize> {
        let eps = 1e-9;
        let lo = xs.partition_point(|&x| x < self.start - eps);
        let hi = xs.partition_point(|&x| x <= self.end + eps);
        if hi.saturating_sub(lo) >= 2 || xs.len() < 2 {
            return lo.min(xs.len())..hi.max(lo.min(xs.len()));
        }
        let lo = lo.min(xs.len() - 2);
        lo..lo + 2
    }
}

impl Drag {
    /// The brush after the pointer moves to fraction `f`.
    #[must_use]
    pub fn apply(self, f: f64, min_span: f64) -> Brush {
        match self.grab {
            Grab::Pan => self.origin.with_edge(Grab::Pan, self.origin.start + (f - self.anchor), min_span),
            g => self.origin.with_edge(g, f, min_span),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: Brush = Brush { start: 0.25, end: 0.75 };

    #[test]
    fn dragging_an_edge_never_crosses_the_other() {
        let (b, d) = B.begin(0.26, 0.03);
        assert_eq!((b, d.grab), (B, Grab::Start));
        assert_eq!(d.apply(0.9, 0.1), Brush { start: 0.65, end: 0.75 });
        assert_eq!(d.apply(-1.0, 0.1).start, 0.0);
        let (_, d) = B.begin(0.74, 0.03);
        assert_eq!(d.grab, Grab::End);
        assert_eq!(d.apply(0.0, 0.1), Brush { start: 0.25, end: 0.35 });
        assert_eq!(d.apply(2.0, 0.1).end, 1.0);
    }

    #[test]
    fn panning_keeps_the_width_and_stops_at_the_ends() {
        let (_, d) = B.begin(0.5, 0.03);
        assert_eq!(d.grab, Grab::Pan);
        let moved = d.apply(0.6, 0.1);
        assert!((moved.start - 0.35).abs() < 1e-12 && (moved.span() - 0.5).abs() < 1e-12);
        assert_eq!(d.apply(5.0, 0.1), Brush { start: 0.5, end: 1.0 });
        assert_eq!(d.apply(-5.0, 0.1), Brush { start: 0.0, end: 0.5 });
    }

    #[test]
    fn pressing_outside_recentres_then_pans() {
        let (b, d) = B.begin(0.95, 0.03);
        assert_eq!(b, Brush { start: 0.5, end: 1.0 });
        assert_eq!(d.grab, Grab::Pan);
        let (b, _) = B.begin(0.0, 0.03);
        assert_eq!(b, Brush { start: 0.0, end: 0.5 });
    }

    #[test]
    fn keys_nudge_edges_within_bounds() {
        assert_eq!(B.key(Grab::Start, "ArrowLeft", false, 0.1, 0.1), Some(Brush { start: 0.15, end: 0.75 }));
        assert_eq!(B.key(Grab::End, "ArrowRight", true, 0.02, 0.1), Some(Brush { start: 0.25, end: 0.95 }));
        assert_eq!(B.key(Grab::Start, "End", false, 0.1, 0.1), Some(Brush { start: 0.65, end: 0.75 }));
        assert_eq!(B.key(Grab::End, "Home", false, 0.1, 0.1), Some(Brush { start: 0.25, end: 0.35 }));
        assert_eq!(Brush::FULL.key(Grab::Start, "ArrowLeft", false, 0.1, 0.1), None, "already at the limit");
        assert_eq!(B.key(Grab::Start, "a", false, 0.1, 0.1), None);
    }

    #[test]
    fn window_maps_to_a_point_range_with_at_least_two_points() {
        let xs = [0.0, 0.25, 0.5, 0.75, 1.0];
        assert_eq!(B.range(&xs), 1..4);
        assert_eq!(Brush::FULL.range(&xs), 0..5);
        assert_eq!(Brush { start: 0.3, end: 0.4 }.range(&xs).len(), 2);
        assert_eq!(Brush { start: 0.95, end: 1.0 }.range(&xs), 3..5);
        assert_eq!(B.range(&[0.5]), 0..1);
        assert_eq!(B.range(&[]), 0..0);
        assert!(Brush::FULL.is_full() && !B.is_full());
    }
}
