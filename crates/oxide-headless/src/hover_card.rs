//! Hover card connector. Anatomy: trigger (a link), content. Every part carries
//! `data-scope="hover-card"`, its `data-part` and `data-state="open|closed"`.
//!
//! The content is a `popover="manual"` in the top layer, tied to the trigger
//! with CSS anchor positioning (see [`crate::overlay`]). Pointer enter and leave
//! on both parts feed the machine, so the card stays open while the pointer
//! moves from the trigger onto it.

pub use oxide_machines::hover_card::*;

pub use crate::overlay::{Align, Placement, Side};
use crate::overlay::{anchor_style, floating_style};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "hover-card";

/// Below the trigger, centred.
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Center);

/// Space between trigger and card.
const GAP_PX: u16 = 8;

/// Attributes for the trigger `<a>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// `aria-expanded`.
    pub aria_expanded: bool,
    /// `aria-controls`: the content id.
    pub aria_controls: String,
    /// Inline style naming the CSS anchor.
    pub style: String,
    /// `data-state`.
    pub data_state: &'static str,
}

/// Attributes for the card content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// Element id, referenced by the trigger's `aria-controls`.
    pub id: String,
    /// Inline style anchoring it to the trigger.
    pub style: String,
    /// `data-side`, for side-aware motion.
    pub data_side: &'static str,
    /// `data-state`.
    pub data_state: &'static str,
}

/// `data-state` for an open flag.
#[must_use]
pub fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// Content id for the instance with id prefix `root`. Adapters show and hide this element.
#[must_use]
pub fn content_id(root: &str) -> String {
    format!("{root}-content")
}

/// Trigger attributes for the instance with id prefix `root`.
#[must_use]
pub fn trigger_attrs(root: &str, state: &HoverCardState) -> TriggerAttrs {
    TriggerAttrs {
        aria_expanded: state.open,
        aria_controls: content_id(root),
        style: anchor_style(root),
        data_state: data_state(state.open),
    }
}

/// Content attributes for the instance with id prefix `root`.
#[must_use]
pub fn content_attrs(root: &str, state: &HoverCardState, placement: Placement) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root),
        style: floating_style(root, placement, GAP_PX),
        data_side: placement.side_str(),
        data_state: data_state(state.open),
    }
}

/// Map a trigger `KeyboardEvent.key` to an event. Only Esc on an open card;
/// `Some` means: prevent default, so an enclosing dialog stays open.
#[must_use]
pub fn keydown(key: &str, state: &HoverCardState) -> Option<HoverCardEvent> {
    (key == "Escape" && state.open).then_some(HoverCardEvent::Escape)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_controls_the_anchored_content() {
        let state = HoverCardState::default();
        let t = trigger_attrs("hc", &state);
        let c = content_attrs("hc", &state, DEFAULT_PLACEMENT);
        assert_eq!(t.aria_controls, c.id);
        assert!(t.style.contains("anchor-name: --hc;"));
        assert!(c.style.contains("position-anchor: --hc;"));
        assert_eq!((c.data_side, c.data_state), ("bottom", "closed"));
    }

    #[test]
    fn escape_only_when_open() {
        let open = HoverCardState { open: true, ..Default::default() };
        assert_eq!(keydown("Escape", &open), Some(HoverCardEvent::Escape));
        assert_eq!(keydown("Escape", &HoverCardState::default()), None);
        assert_eq!(keydown("Enter", &open), None);
    }
}
