//! Carousel connector. Anatomy: root, viewport, slide, prev, next, dots, dot,
//! play. Every part carries `data-scope="carousel"` and its `data-part`. The
//! viewport is a scroll-snap container, so touch swipes, trackpads and
//! wheel scrolling work without script; the adapter keeps the index in step
//! with where it settles.

pub use hydroxy_machines::carousel::*;

pub use hydroxy_machines::ChangeReason;

use crate::Orientation;
use crate::attrs::nav_key;
use hydroxy_virtual::Nav;

/// The `data-scope` value on every carousel part.
pub const SCOPE: &str = "carousel";

/// Attributes for a slide (`role="group"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideAttrs {
    /// `id`, so the adapter can scroll it into view.
    pub id: String,
    /// `aria-label`: `1 of 5`.
    pub aria_label: String,
    /// `inert` and `aria-hidden` for slides other than the current one, so
    /// off-screen content stays out of the tab order and the reading order.
    pub inert: bool,
    /// `data-state`: `current` or `idle`.
    pub data_state: &'static str,
}

/// The slide's id.
#[must_use]
pub fn slide_id(root: &str, i: usize) -> String {
    format!("{root}-slide-{i}")
}

/// The viewport's id.
#[must_use]
pub fn viewport_id(root: &str) -> String {
    format!("{root}-viewport")
}

/// Attributes for slide `i`.
#[must_use]
pub fn slide_attrs(root: &str, state: &CarouselState, input: &CarouselInput, i: usize) -> SlideAttrs {
    let current = state.index.value == i;
    SlideAttrs {
        id: slide_id(root, i),
        aria_label: format!("{} of {}", i + 1, input.count),
        inert: !current,
        data_state: if current { "current" } else { "idle" },
    }
}

/// Whether Previous is disabled: at the first slide of a carousel that does not loop.
#[must_use]
pub fn prev_disabled(state: &CarouselState, input: &CarouselInput) -> bool {
    input.count < 2 || (!input.looping && state.index.value == 0)
}

/// Whether Next is disabled: at the last slide of a carousel that does not loop.
#[must_use]
pub fn next_disabled(state: &CarouselState, input: &CarouselInput) -> bool {
    input.count < 2 || (!input.looping && state.index.value + 1 >= input.count)
}

/// `aria-live` for the viewport: quiet while autoplay is moving it, polite otherwise.
#[must_use]
pub fn aria_live(state: &CarouselState) -> &'static str {
    if state.running() { "off" } else { "polite" }
}

/// The slide a scroll position has settled on.
#[must_use]
pub fn scroll_index(scroll_left: f64, slide_width: f64, count: usize) -> usize {
    if slide_width <= 0.0 || count == 0 {
        return 0;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rounded, then clamped to the slide count"
    )]
    let at = (scroll_left.abs() / slide_width).round() as usize;
    at.min(count - 1)
}

/// The event for a key on the carousel, if the key does anything.
#[must_use]
pub fn key_event(key: &str, count: usize) -> Option<CarouselEvent> {
    match nav_key(key, Some(Orientation::Horizontal))? {
        Nav::Prev => Some(CarouselEvent::Prev(ChangeReason::Keyboard)),
        Nav::Next => Some(CarouselEvent::Next(ChangeReason::Keyboard)),
        Nav::First => Some(CarouselEvent::Go(0, ChangeReason::Keyboard)),
        Nav::Last => Some(CarouselEvent::Go(count.saturating_sub(1), ChangeReason::Keyboard)),
        _ => None,
    }
}

/// Inner markup of the pause icon: two bars.
pub const PAUSE_MARKUP: &str = r#"<svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" aria-hidden="true"><rect x="6" y="5" width="4" height="14" rx="1"/><rect x="14" y="5" width="4" height="14" rx="1"/></svg>"#;

/// Inner markup of the play icon: a triangle.
pub const PLAY_MARKUP: &str = r#"<svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" aria-hidden="true"><path d="M7 4.5v15a1 1 0 0 0 1.5.86l12-7.5a1 1 0 0 0 0-1.72l-12-7.5A1 1 0 0 0 7 4.5Z"/></svg>"#;

/// The accessible name of the play / pause button.
#[must_use]
pub fn play_label(state: &CarouselState) -> &'static str {
    if state.playing { "Pause slide show" } else { "Start slide show" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Controllable;

    fn input() -> CarouselInput {
        CarouselInput { count: 3, ..CarouselInput::default() }
    }

    #[test]
    fn only_the_current_slide_is_reachable() {
        let s = CarouselState { index: Controllable::new(1, false), ..CarouselState::default() };
        let a = slide_attrs("c", &s, &input(), 1);
        assert!(!a.inert && a.aria_label == "2 of 3" && a.data_state == "current");
        assert!(slide_attrs("c", &s, &input(), 0).inert);
    }

    #[test]
    fn ends_disable_buttons_unless_looping() {
        let first = CarouselState::default();
        assert!(prev_disabled(&first, &input()) && !next_disabled(&first, &input()));
        let looping = CarouselInput { looping: true, ..input() };
        assert!(!prev_disabled(&first, &looping));
    }

    #[test]
    fn scroll_position_rounds_to_the_nearest_slide() {
        assert_eq!(scroll_index(0.0, 300.0, 3), 0);
        assert_eq!(scroll_index(160.0, 300.0, 3), 1);
        assert_eq!(scroll_index(9000.0, 300.0, 3), 2);
        assert_eq!(scroll_index(-300.0, 300.0, 3), 1, "RTL scroll offsets are negative");
        assert_eq!(key_event("End", 4), Some(CarouselEvent::Go(3, ChangeReason::Keyboard)));
        assert_eq!(key_event("a", 4), None);
    }
}
