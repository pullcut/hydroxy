//! Popover connector. Anatomy: trigger, content, close. Every part carries
//! `data-scope="popover"` and its `data-part`.
//!
//! Like the menu, the trigger names the content with `popovertarget`, so clicking
//! it never counts as an outside click; the adapter cancels the click's default
//! and drives the popover from the machine.

pub use hydroxy_machines::popover::*;

use crate::overlay::{self, Align, Placement, Side};

/// The `data-scope` value on every popover part.
pub const SCOPE: &str = "popover";

/// Below the trigger, centred on it.
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Center);

/// Pixels between trigger and panel.
pub const GAP_PX: u16 = 8;

/// What opening focuses inside the content: the first match, else the content itself.
pub const FOCUSABLE: &str = "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), \
     textarea:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";

/// Attributes for the trigger (`<button>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// `id`, referenced by the content's `aria-labelledby`.
    pub id: String,
    /// `aria-expanded`.
    pub aria_expanded: bool,
    /// `aria-controls` and `popovertarget`: the content's id.
    pub aria_controls: String,
    /// `style`: names the CSS anchor.
    pub style: String,
    /// `data-state`: `"open"` or `"closed"`.
    pub data_state: &'static str,
}

/// Attributes for the content (`role="dialog"`, `popover="auto"`, `tabindex="-1"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// `id`, referenced by the trigger.
    pub id: String,
    /// `aria-labelledby`: the trigger's id.
    pub aria_labelledby: String,
    /// `style`: anchored placement with flip.
    pub style: String,
    /// `data-side`, for side-aware motion.
    pub data_side: &'static str,
    /// `data-state`.
    pub data_state: &'static str,
}

/// The trigger's id.
#[must_use]
pub fn trigger_id(root: &str) -> String {
    format!("{root}-trigger")
}

/// The content's id.
#[must_use]
pub fn content_id(root: &str) -> String {
    format!("{root}-content")
}

fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// Attributes for the trigger.
#[must_use]
pub fn trigger_attrs(root: &str, state: &PopoverState) -> TriggerAttrs {
    TriggerAttrs {
        id: trigger_id(root),
        aria_expanded: state.open,
        aria_controls: content_id(root),
        style: overlay::anchor_style(root),
        data_state: data_state(state.open),
    }
}

/// Attributes for the content.
#[must_use]
pub fn content_attrs(root: &str, state: &PopoverState, placement: Placement) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root),
        aria_labelledby: trigger_id(root),
        style: overlay::floating_style(root, placement, GAP_PX),
        data_side: placement.side_str(),
        data_state: data_state(state.open),
    }
}

/// Keys inside the open panel. Escape is handled here (not left to the browser)
/// so focus goes back to the trigger.
#[must_use]
pub fn content_keydown(key: &str) -> Option<PopoverEvent> {
    (key == "Escape").then_some(PopoverEvent::Close { restore_focus: true })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_and_content_reference_each_other() {
        let state = PopoverState { open: true, controlled: false };
        let trigger = trigger_attrs("p", &state);
        let content = content_attrs("p", &state, DEFAULT_PLACEMENT);
        assert_eq!(trigger.aria_controls, content.id);
        assert_eq!(content.aria_labelledby, trigger.id);
        assert!(trigger.aria_expanded);
        assert!(content.style.contains("position-area: block-end center;"));
        assert!(content.style.contains("margin-block-start: 8px;"));
    }

    #[test]
    fn escape_closes_and_restores_focus() {
        assert_eq!(content_keydown("Escape"), Some(PopoverEvent::Close { restore_focus: true }));
        assert_eq!(content_keydown("Tab"), None);
    }
}
