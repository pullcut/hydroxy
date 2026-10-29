//! Context menu connector. Anatomy: trigger (the area that takes the right
//! click), anchor (a zero-size point at the pointer), content, item.
//! Every part carries `data-scope="context-menu"` and its `data-part`.
//!
//! Behaviour is the menu's: the same machine, item attributes and keys. The
//! difference is how it opens (right click, Shift+F10 or the `ContextMenu` key)
//! and where: the content anchors to a point, not to a button. The point is a
//! fixed, zero-size anchor element, so CSS anchor positioning still flips the
//! menu when it would run off the viewport.

pub use crate::menu::{
    Direction, Highlight, ItemAttrs, MenuCommand, MenuEntry, MenuEvent, MenuInput, MenuMachine, MenuState,
    content_keydown, item_attrs, item_id,
};
use hydroxy_machines::Modifiers;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "context-menu";

/// Attributes for the content (`popover="auto"`, `role="menu"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// `id`, so the adapter can show and hide it.
    pub id: String,
    /// `style`: anchored below and after the point, flipping on either axis.
    pub style: String,
    /// `data-state`.
    pub data_state: &'static str,
}

/// The content's id.
#[must_use]
pub fn content_id(root: &str) -> String {
    format!("{root}-content")
}

/// `data-state` for an open flag.
#[must_use]
pub fn data_state(open: bool) -> &'static str {
    crate::attrs::data_state(open)
}

/// Inline style for the anchor point at viewport coordinates `(x, y)`.
#[must_use]
pub fn anchor_style(root: &str, x: f64, y: f64) -> String {
    format!(
        "position: fixed; left: {x}px; top: {y}px; width: 0; height: 0; pointer-events: none; anchor-name: --{root};"
    )
}

/// Attributes for the content.
#[must_use]
pub fn content_attrs(root: &str, state: &MenuState) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root),
        style: format!(
            "position: fixed; inset: auto; margin: 0; position-anchor: --{root}; position-area: block-end span-inline-end; position-try-fallbacks: flip-block, flip-inline, flip-block flip-inline;"
        ),
        data_state: data_state(state.open.value),
    }
}

/// Whether a keydown on the trigger area opens the menu: the `ContextMenu` key,
/// or Shift+F10. Prevent the default when it does.
#[must_use]
pub fn opens_from_key(key: &str, mods: Modifiers) -> bool {
    key == "ContextMenu" || (mods.shift && key == "F10")
}

/// Where a `contextmenu` event should open the menu. Keyboard-fired events can
/// report `(0, 0)`; then use the focused element's bottom-left, `fallback`.
#[must_use]
pub fn open_point(client: (f64, f64), fallback: (f64, f64)) -> (f64, f64) {
    if client == (0.0, 0.0) { fallback } else { client }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_anchors_to_the_point() {
        let content = content_attrs("cm", &MenuState::default());
        assert!(content.style.contains("position-anchor: --cm;"));
        assert!(content.style.contains("flip-inline"));
        let anchor = anchor_style("cm", 12.0, 40.5);
        assert!(anchor.contains("left: 12px; top: 40.5px;"));
        assert!(anchor.contains("anchor-name: --cm;"));
    }

    #[test]
    fn keyboard_openers_and_point_fallback() {
        assert!(opens_from_key("ContextMenu", Modifiers::NONE));
        assert!(opens_from_key("F10", Modifiers::SHIFT));
        assert!(!opens_from_key("F10", Modifiers::NONE));
        assert_eq!(open_point((0.0, 0.0), (5.0, 9.0)), (5.0, 9.0));
        assert_eq!(open_point((30.0, 2.0), (5.0, 9.0)), (30.0, 2.0));
    }
}
