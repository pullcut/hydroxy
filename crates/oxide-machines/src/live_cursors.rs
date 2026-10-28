//! Live cursors: other people's pointers over a shared surface. The host owns the
//! network and hands over positions in normalised coordinates (0–1 across the
//! surface). This module clamps them, and tracks when each cursor last moved so
//! still ones can fade out.
//!
//! Time is counted in ticks of a fixed interval the adapter runs, so no wall
//! clock is needed on any platform.

use std::collections::HashMap;

/// Clamp a normalised coordinate to the surface; NaN goes to 0.
#[must_use]
pub fn clamp_unit(v: f64) -> f64 {
    if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) }
}

/// A pointer at `(px, py)` inside a `width` × `height` box, as normalised
/// coordinates. `None` for an empty box.
#[must_use]
pub fn normalise(px: f64, py: f64, width: f64, height: f64) -> Option<(f64, f64)> {
    (width > 0.0 && height > 0.0).then(|| (clamp_unit(px / width), clamp_unit(py / height)))
}

/// Where each cursor was and the tick it last moved on.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorTracker {
    seen: HashMap<String, ((f64, f64), u64)>,
    tick: u64,
}

impl CursorTracker {
    /// The current tick.
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Record the latest positions. A cursor whose position changed counts as moved now;
    /// one that is gone is forgotten.
    pub fn update<'a>(&mut self, positions: impl IntoIterator<Item = (&'a str, f64, f64)>) {
        let tick = self.tick;
        let mut next = HashMap::new();
        for (id, x, y) in positions {
            let pos = (clamp_unit(x), clamp_unit(y));
            let moved = match self.seen.get(id) {
                Some((old, at)) if *old == pos => *at,
                _ => tick,
            };
            next.insert(id.to_string(), (pos, moved));
        }
        self.seen = next;
    }

    /// Advance the clock by one tick.
    pub fn advance(&mut self) {
        self.tick += 1;
    }

    /// Whether `id` has been still for at least `idle_ticks`. Unknown ids are not idle.
    #[must_use]
    pub fn is_idle(&self, id: &str, idle_ticks: u64) -> bool {
        self.seen.get(id).is_some_and(|(_, at)| self.tick.saturating_sub(*at) >= idle_ticks)
    }
}

/// Inline style placing a cursor. The cursor box is as big as the surface, so a
/// percentage translate is a percentage of the surface; CSS smooths the moves.
#[must_use]
pub fn position_style(x: f64, y: f64) -> String {
    format!("transform: translate({:.3}%, {:.3}%);", clamp_unit(x) * 100.0, clamp_unit(y) * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_and_normalises() {
        assert!((clamp_unit(1.4) - 1.0).abs() < f64::EPSILON);
        assert!(clamp_unit(f64::NAN).abs() < f64::EPSILON);
        assert_eq!(normalise(50.0, 25.0, 100.0, 100.0), Some((0.5, 0.25)));
        assert_eq!(normalise(5.0, 5.0, 0.0, 10.0), None);
    }

    #[test]
    fn still_cursors_go_idle_and_moving_ones_wake() {
        let mut t = CursorTracker::default();
        t.update([("ana", 0.1, 0.1), ("ben", 0.5, 0.5)]);
        for _ in 0..3 {
            t.advance();
        }
        // Ana moves, Ben stays put.
        t.update([("ana", 0.2, 0.1), ("ben", 0.5, 0.5)]);
        assert!(!t.is_idle("ana", 3));
        assert!(t.is_idle("ben", 3));
        assert!(!t.is_idle("nobody", 0));
    }

    #[test]
    fn gone_cursors_are_forgotten() {
        let mut t = CursorTracker::default();
        t.update([("ana", 0.1, 0.1)]);
        t.advance();
        t.update([]);
        t.update([("ana", 0.1, 0.1)]);
        assert!(!t.is_idle("ana", 1), "a returning cursor starts fresh");
    }

    #[test]
    fn style_is_a_percentage_translate() {
        assert_eq!(position_style(0.5, 2.0), "transform: translate(50.000%, 100.000%);");
    }
}
