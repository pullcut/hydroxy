//! Menu connector. Anatomy: trigger, content, label, item. Every part carries
//! `data-scope="menu"` and its `data-part`.
//!
//! The trigger names the content with `popovertarget`, so clicking it never
//! counts as an outside click. The adapter cancels the click's default and drives
//! the popover from the machine instead.

pub use oxide_machines::menu::*;

use crate::overlay::{self, Align, Placement, Side};

/// The `data-scope` value on every menu part.
pub const SCOPE: &str = "menu";

/// Below the trigger, lined up with its start edge.
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

/// Pixels between trigger and menu.
pub const GAP_PX: u16 = 6;

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

/// Attributes for the content (`popover="auto"`, `role="menu"`).
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

/// Attributes for an item (`<button role="menuitem" tabindex="-1">`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttrs {
    /// `id`, so the adapter can focus it.
    pub id: String,
    /// Native `disabled`.
    pub disabled: bool,
    /// Emit `data-highlighted` when true.
    pub highlighted: bool,
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

/// The id of item `value`.
#[must_use]
pub fn item_id(root: &str, value: &str) -> String {
    format!("{root}-item-{value}")
}

fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// Attributes for the trigger.
#[must_use]
pub fn trigger_attrs(root: &str, state: &MenuState) -> TriggerAttrs {
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
pub fn content_attrs(root: &str, state: &MenuState, placement: Placement) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root),
        aria_labelledby: trigger_id(root),
        style: overlay::floating_style(root, placement, GAP_PX),
        data_side: placement.side_str(),
        data_state: data_state(state.open),
    }
}

/// Attributes for item `value`.
#[must_use]
pub fn item_attrs(root: &str, state: &MenuState, input: &MenuInput, value: &str) -> ItemAttrs {
    ItemAttrs {
        id: item_id(root, value),
        disabled: input.items.iter().any(|i| i.value == value && i.disabled),
        highlighted: state.highlighted.as_deref() == Some(value),
    }
}

/// Keys on the trigger. Enter and Space are left to the button's native click.
#[must_use]
pub fn trigger_keydown(key: &str) -> Option<MenuEvent> {
    match key {
        "ArrowDown" => Some(MenuEvent::Open(Highlight::First)),
        "ArrowUp" => Some(MenuEvent::Open(Highlight::Last)),
        _ => None,
    }
}

/// Keys inside the open menu. The `bool` says whether to prevent the default:
/// Tab closes but still moves focus. Enter and Space reach items as native clicks.
#[must_use]
pub fn content_keydown(key: &str) -> Option<(MenuEvent, bool)> {
    match key {
        "ArrowDown" => Some((MenuEvent::Move(Direction::Next), true)),
        "ArrowUp" => Some((MenuEvent::Move(Direction::Prev), true)),
        "Home" => Some((MenuEvent::Home, true)),
        "End" => Some((MenuEvent::End, true)),
        "Escape" => Some((MenuEvent::Close { restore_focus: true }, true)),
        "Tab" => Some((MenuEvent::Close { restore_focus: false }, false)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_and_content_reference_each_other() {
        let state = MenuState::default();
        let trigger = trigger_attrs("m", &state);
        let content = content_attrs("m", &state, DEFAULT_PLACEMENT);
        assert_eq!(trigger.aria_controls, content.id);
        assert_eq!(content.aria_labelledby, trigger.id);
        assert!(content.style.contains("position-anchor: --m;"));
        assert_eq!(trigger.style, "anchor-name: --m;");
    }

    #[test]
    fn tab_closes_without_preventing_default() {
        assert_eq!(content_keydown("Tab"), Some((MenuEvent::Close { restore_focus: false }, false)));
        assert_eq!(content_keydown("Escape"), Some((MenuEvent::Close { restore_focus: true }, true)));
        assert_eq!(trigger_keydown("Enter"), None);
    }
}
