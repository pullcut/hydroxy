//! Message scroller connector. Anatomy: root, viewport (the scrolling `role="log"`),
//! and a jump button that appears when the reader is away from the bottom.
//! Every part carries `data-scope="message-scroller"` and its `data-part`.
//!
//! The viewport is the one scroll container, so keyboard scrolling is native. It is
//! focusable so keyboard users can scroll it.

pub use hydroxy_machines::message_scroller::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "message-scroller";

/// Attributes for the viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportAttrs {
    /// Always `"log"`: new messages are announced politely.
    pub role: &'static str,
    /// Always `"polite"`.
    pub aria_live: &'static str,
    /// Always `"0"`: reachable by keyboard so it can scroll.
    pub tabindex: &'static str,
    /// `"following"` at the bottom, `"away"` otherwise.
    pub data_state: &'static str,
}

/// Distance from the bottom edge in px, from the scroll geometry.
#[must_use]
pub fn from_bottom(scroll_height: f64, scroll_top: f64, client_height: f64) -> f64 {
    (scroll_height - scroll_top - client_height).max(0.0)
}

/// `data-state` for every part.
#[must_use]
pub fn data_state(state: &MessageScrollerState) -> &'static str {
    if state.stuck { "following" } else { "away" }
}

/// Attributes for the viewport.
#[must_use]
pub fn viewport_attrs(state: &MessageScrollerState) -> ViewportAttrs {
    ViewportAttrs { role: "log", aria_live: "polite", tabindex: "0", data_state: data_state(state) }
}

/// Whether the jump button shows.
#[must_use]
pub fn jump_visible(state: &MessageScrollerState) -> bool {
    !state.stuck
}

/// Accessible name and text of the jump button.
#[must_use]
pub fn jump_label(unread: usize) -> String {
    match unread {
        0 => "Jump to latest".to_string(),
        1 => "1 new message".to_string(),
        n => format!("{n} new messages"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_never_negative() {
        assert!((from_bottom(1000.0, 600.0, 400.0)).abs() < f64::EPSILON);
        assert!((from_bottom(1000.0, 100.0, 400.0) - 500.0).abs() < f64::EPSILON);
        assert!((from_bottom(1000.0, 700.0, 400.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn jump_label_counts() {
        assert_eq!(jump_label(0), "Jump to latest");
        assert_eq!(jump_label(1), "1 new message");
        assert_eq!(jump_label(3), "3 new messages");
    }

    #[test]
    fn button_shows_only_when_away() {
        let mut s = MessageScrollerState { stuck: true, ..MessageScrollerState::default() };
        assert!(!jump_visible(&s));
        s.stuck = false;
        assert!(jump_visible(&s));
        assert_eq!(viewport_attrs(&s).data_state, "away");
    }
}
