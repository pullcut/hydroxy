//! Citation popover connector. Anatomy: text (the wrapper for cited prose),
//! cited (a cited run), marker (a `<sup>` link "[n]" to the source, the hover
//! card's trigger, described by the card), card (a `popover="manual"` anchored
//! to the marker), favicon (an image, or the host's initial), title, host,
//! snippet (with the quote in a `<mark>`), open (a link to the source). Every
//! part carries `data-scope="citation"` and its `data-part`.
//!
//! Timing is the hover card machine's: hover opens after a short delay, focus
//! opens at once, moving onto the card keeps it open, Esc closes it. Focus moving
//! into the card (Tab from the marker reaches its link next) counts as staying.

pub use hydroxy_machines::citation_popover::*;
pub use hydroxy_machines::hover_card::{
    HoverCardCommand, HoverCardEvent, HoverCardInput, HoverCardMachine, HoverCardState,
};

use crate::overlay::{Align, Placement, Side, anchor_style, floating_style};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "citation";

/// Ids for one marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The marker link; also the CSS anchor name.
    pub marker: String,
    /// The card.
    pub card: String,
    /// The snippet, which describes the marker.
    pub snippet: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { marker: format!("{root}-marker"), card: format!("{root}-card"), snippet: format!("{root}-snippet") }
}

/// The hover card input citations use: quick to open, a little slower to close
/// so the pointer can travel to the card.
#[must_use]
pub fn input() -> HoverCardInput {
    HoverCardInput { open: None, default_open: false, open_delay_ms: 200, close_delay_ms: 200 }
}

/// Inline style naming the marker as the card's anchor.
#[must_use]
pub fn marker_style(ids: &Ids) -> String {
    anchor_style(&ids.marker)
}

/// Inline style placing the card below the marker, flipping above when needed.
#[must_use]
pub fn card_style(ids: &Ids) -> String {
    floating_style(&ids.marker, Placement::new(Side::Bottom, Align::Center), 6)
}

/// Map a key on the marker to an event: Esc closes an open card.
#[must_use]
pub fn keydown(key: &str, state: &HoverCardState) -> Option<HoverCardEvent> {
    (key == "Escape" && state.open).then_some(HoverCardEvent::Escape)
}

/// The marker's visible text: "1" for the first source (brackets come from CSS).
#[must_use]
pub fn marker_text(index: usize) -> String {
    (index + 1).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_is_anchored_to_its_marker() {
        let i = ids("c1");
        assert!(card_style(&i).contains("position-anchor: --c1-marker;"));
        assert!(marker_style(&i).contains("anchor-name: --c1-marker;"));
        assert_eq!(marker_text(0), "1");
        let open = HoverCardState { open: true, ..HoverCardState::default() };
        assert_eq!(keydown("Escape", &open), Some(HoverCardEvent::Escape));
        assert_eq!(keydown("Escape", &HoverCardState::default()), None);
    }
}
