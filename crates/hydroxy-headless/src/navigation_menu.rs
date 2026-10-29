//! Navigation menu connector. Anatomy: root (`<nav>`), list, item (`<li>`),
//! trigger (a button), link, content (the flyout panel) and panel-link. Every
//! part carries `data-scope="navigation-menu"` and its `data-part`.
//!
//! A panel is a `popover="auto"` in the top layer, anchored under its trigger.
//! Triggers name their panel with `popovertarget`, so clicking one never counts
//! as an outside click. Pointer enter and leave on the nav and the panel feed
//! the machine's delays.

pub use hydroxy_machines::navigation_menu::*;

use crate::Orientation;
use crate::attrs::{data_state, nav_key};
use crate::overlay::{self, Align, Placement, Side};
use hydroxy_virtual::Nav;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "navigation-menu";

/// Below the trigger, lined up with its start edge.
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

/// Pixels between trigger and panel.
pub const GAP_PX: u16 = 8;

/// Attributes for a panel trigger (`<button>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// `aria-expanded`.
    pub aria_expanded: bool,
    /// `aria-controls` and `popovertarget`: the panel's id.
    pub aria_controls: String,
    /// `style`: names the CSS anchor.
    pub style: String,
    /// `data-state`: `"open"` or `"closed"`.
    pub data_state: &'static str,
}

/// Attributes for a panel (`popover="auto"`).
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

/// Id of the focusable element (trigger or link) for item `value`.
#[must_use]
pub fn item_id(root: &str, value: &str) -> String {
    format!("{root}-{value}-item")
}

/// Id of the panel for item `value`.
#[must_use]
pub fn content_id(root: &str, value: &str) -> String {
    format!("{root}-{value}-content")
}

fn anchor_name(root: &str, value: &str) -> String {
    format!("{root}-{value}")
}

/// Attributes for the trigger of item `value`.
#[must_use]
pub fn trigger_attrs(root: &str, state: &NavigationMenuState, value: &str) -> TriggerAttrs {
    let open = state.open.value.as_deref() == Some(value);
    TriggerAttrs {
        aria_expanded: open,
        aria_controls: content_id(root, value),
        style: overlay::anchor_style(&anchor_name(root, value)),
        data_state: data_state(open),
    }
}

/// Attributes for the panel of item `value`.
#[must_use]
pub fn content_attrs(root: &str, state: &NavigationMenuState, value: &str, placement: Placement) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root, value),
        aria_labelledby: item_id(root, value),
        style: overlay::floating_style(&anchor_name(root, value), placement, GAP_PX),
        data_side: placement.side_str(),
        data_state: data_state(state.open.value.as_deref() == Some(value)),
    }
}

/// Keys on a top-level item (trigger or link). `Some` means prevent default.
/// Enter and Space are left to the native click.
#[must_use]
pub fn item_keydown(key: &str, value: &str, has_panel: bool) -> Option<NavigationMenuEvent> {
    let from = value.to_string();
    match nav_key(key, Some(Orientation::Horizontal)) {
        Some(Nav::Next) => Some(NavigationMenuEvent::Move { from, direction: Direction::Next }),
        Some(Nav::Prev) => Some(NavigationMenuEvent::Move { from, direction: Direction::Prev }),
        Some(Nav::First) => Some(NavigationMenuEvent::Home),
        Some(Nav::Last) => Some(NavigationMenuEvent::End),
        _ => (key == "ArrowDown" && has_panel).then_some(NavigationMenuEvent::Open(from)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Controllable;

    #[test]
    fn trigger_and_panel_reference_each_other() {
        let state = NavigationMenuState { open: Controllable::new(Some("a".into()), false), ..Default::default() };
        let t = trigger_attrs("nav", &state, "a");
        let c = content_attrs("nav", &state, "a", DEFAULT_PLACEMENT);
        assert_eq!(t.aria_controls, c.id);
        assert_eq!(c.aria_labelledby, item_id("nav", "a"));
        assert!(c.style.contains("position-anchor: --nav-a;"));
        assert_eq!((t.data_state, c.data_state), ("open", "open"));
        assert_eq!(trigger_attrs("nav", &state, "b").data_state, "closed");
    }

    #[test]
    fn keys_map_to_events() {
        assert_eq!(
            item_keydown("ArrowRight", "a", true),
            Some(NavigationMenuEvent::Move { from: "a".into(), direction: Direction::Next })
        );
        assert_eq!(item_keydown("ArrowDown", "a", true), Some(NavigationMenuEvent::Open("a".into())));
        assert_eq!(item_keydown("ArrowDown", "a", false), None);
        assert_eq!(item_keydown("Home", "a", false), Some(NavigationMenuEvent::Home));
    }
}
