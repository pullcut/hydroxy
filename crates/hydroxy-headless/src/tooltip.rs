//! Tooltip connector. Anatomy: trigger, content. Every part carries
//! `data-scope="tooltip"`, its `data-part` and `data-state="open|closed"`.
//!
//! The content is a `popover="manual"` in the top layer, tied to the trigger
//! with CSS anchor positioning (see [`crate::overlay`]). Adapters show and hide
//! it to match the machine's open state.

pub use hydroxy_machines::tooltip::*;

pub use crate::overlay::{Align, Placement, Side};
use crate::overlay::{anchor_style, floating_style};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "tooltip";

/// Above the trigger, centred.
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Top, Align::Center);

/// Space between trigger and tooltip.
const GAP_PX: u16 = 8;

/// Attributes for the trigger `<button>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// Element id.
    pub id: String,
    /// `aria-describedby`: the content id, so the tooltip text is read with the trigger.
    pub aria_describedby: String,
    /// Inline style naming the CSS anchor.
    pub style: String,
    /// `data-state`.
    pub data_state: &'static str,
}

/// Attributes for the tooltip content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// Element id, referenced by the trigger's `aria-describedby`.
    pub id: String,
    /// Always `tooltip`.
    pub role: &'static str,
    /// Always `manual`: the machine decides when it shows.
    pub popover: &'static str,
    /// Inline style anchoring it to the trigger.
    pub style: String,
    /// `data-side`, for side-aware styling.
    pub data_side: &'static str,
    /// `data-state`.
    pub data_state: &'static str,
}

/// `data-state` for an open flag.
#[must_use]
pub fn data_state(open: bool) -> &'static str {
    crate::attrs::data_state(open)
}

/// Content id for the instance with id prefix `root`. Adapters show and hide this element.
#[must_use]
pub fn content_id(root: &str) -> String {
    format!("{root}-content")
}

/// Trigger attributes for the instance with id prefix `root`.
#[must_use]
pub fn trigger_attrs(root: &str, state: &TooltipState) -> TriggerAttrs {
    TriggerAttrs {
        id: format!("{root}-trigger"),
        aria_describedby: content_id(root),
        style: anchor_style(root),
        data_state: data_state(state.open),
    }
}

/// Content attributes for the instance with id prefix `root`.
#[must_use]
pub fn content_attrs(root: &str, state: &TooltipState, placement: Placement) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root),
        role: "tooltip",
        popover: "manual",
        style: floating_style(root, placement, GAP_PX),
        data_side: placement.side_str(),
        data_state: data_state(state.open),
    }
}

/// Map a trigger `KeyboardEvent.key` to an event. Only Esc on an open tooltip;
/// `Some` means: prevent default, so an enclosing dialog stays open.
#[must_use]
pub fn keydown(key: &str, state: &TooltipState) -> Option<TooltipEvent> {
    (key == "Escape" && state.open).then_some(TooltipEvent::Escape)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_describes_itself_with_the_anchored_content() {
        let state = TooltipState::default();
        let t = trigger_attrs("tt", &state);
        let c = content_attrs("tt", &state, DEFAULT_PLACEMENT);
        assert_eq!(t.aria_describedby, c.id);
        assert!(t.style.contains("anchor-name: --tt;"));
        assert!(c.style.contains("position-anchor: --tt;"));
        assert_eq!((c.role, c.popover, c.data_side), ("tooltip", "manual", "top"));
    }

    #[test]
    fn escape_only_when_open() {
        let open = TooltipState { open: true, controlled: false };
        assert_eq!(keydown("Escape", &open), Some(TooltipEvent::Escape));
        assert_eq!(keydown("Escape", &TooltipState::default()), None);
        assert_eq!(keydown("Enter", &open), None);
    }
}
