//! Live cursors connector. Anatomy: root (the shared surface, `position:
//! relative`), content (the host's children), layer (`aria-hidden`, covers the
//! surface), cursor (one per person, as big as the surface and translated by a
//! percentage, so CSS can smooth every move), arrow, label. Every part carries
//! `data-scope="live-cursors"` and its `data-part`; cursors carry `data-idle`.
//!
//! The layer is hidden from assistive tech: a pointer position means nothing
//! read aloud. Pair it with a presence bar, which says who is here.

pub use oxide_machines::live_cursors::*;

use crate::jewel::Jewel;
use crate::presence_bar::jewel_for;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "live-cursors";

/// How often the idle clock ticks, in milliseconds.
pub const TICK_MS: u32 = 500;

/// Someone else's pointer, as the host last heard it.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteCursor {
    /// Stable id.
    pub id: String,
    /// Shown on the label.
    pub name: String,
    /// Across the surface, 0 (left) to 1 (right).
    pub x: f64,
    /// Down the surface, 0 (top) to 1 (bottom).
    pub y: f64,
    /// Tint. `None` picks one from the id, matching the presence bar.
    pub jewel: Option<Jewel>,
}

impl RemoteCursor {
    /// A cursor at `(x, y)`.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, x: f64, y: f64) -> Self {
        Self { id: id.into(), name: name.into(), x, y, jewel: None }
    }

    /// The same cursor with this tint.
    #[must_use]
    pub fn jewel(mut self, jewel: Jewel) -> Self {
        self.jewel = Some(jewel);
        self
    }

    /// The tint to draw with.
    #[must_use]
    pub fn tint(&self) -> Jewel {
        self.jewel.unwrap_or_else(|| jewel_for(&self.id))
    }
}

/// Ticks of stillness after which a cursor fades, from milliseconds.
#[must_use]
pub fn idle_ticks(idle_ms: u32) -> u64 {
    u64::from(idle_ms.div_ceil(TICK_MS).max(1))
}

/// The SVG path of the arrow, drawn in a 16 × 16 box with the tip at the origin.
pub const ARROW_PATH: &str = "M1 1 L1 13.5 L4.6 10.2 L7.1 15.2 L9.4 14.1 L6.9 9.2 L11.8 9.2 Z";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_time_rounds_up_to_ticks() {
        assert_eq!(idle_ticks(3_000), 6);
        assert_eq!(idle_ticks(1_200), 3);
        assert_eq!(idle_ticks(0), 1);
    }

    #[test]
    fn cursor_tint_matches_presence() {
        assert_eq!(RemoteCursor::new("u1", "Ana", 0.0, 0.0).tint(), jewel_for("u1"));
    }
}
