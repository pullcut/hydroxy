//! Menubar connector. Anatomy: root, trigger (one per menu), then each menu's
//! content and items, which are the [`crate::menu`] parts. The root and the
//! triggers carry `data-scope="menubar"`; content and items keep
//! `data-scope="menu"` so they share the menu's look.
//!
//! Each menu is a [`crate::menu`] instance with id prefix [`menu_root`],
//! driven open or closed by the menubar machine.

pub use oxide_machines::menubar::*;

/// The `data-scope` value on the root and the triggers.
pub const SCOPE: &str = "menubar";

/// Attributes for a trigger (`<button role="menuitem">`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// `tabindex`: `0` on the one tab stop, `-1` on the rest.
    pub tabindex: &'static str,
    /// Native `disabled`.
    pub disabled: bool,
    /// `data-state`: `"open"` or `"closed"`.
    pub data_state: &'static str,
}

/// The id prefix of the menu `value` inside the menubar with id prefix `root`.
#[must_use]
pub fn menu_root(root: &str, value: &str) -> String {
    format!("{root}-{value}")
}

/// Attributes for the trigger of menu `value`.
#[must_use]
pub fn trigger_attrs(state: &MenubarState, input: &MenubarInput, value: &str) -> TriggerAttrs {
    let open = state.open.as_deref() == Some(value);
    let tab_stop = state.active.as_deref() == Some(value);
    TriggerAttrs {
        tabindex: if tab_stop { "0" } else { "-1" },
        disabled: input.menus.iter().any(|m| m.value == value && m.disabled),
        data_state: if open { "open" } else { "closed" },
    }
}

/// Keys on the root, from a trigger or from inside an open menu. `Some` means prevent default.
#[must_use]
pub fn root_keydown(key: &str) -> Option<MenubarEvent> {
    match key {
        "ArrowRight" => Some(MenubarEvent::Move(Direction::Next)),
        "ArrowLeft" => Some(MenubarEvent::Move(Direction::Prev)),
        _ => None,
    }
}

/// Keys on the trigger of menu `value`. `Some` means prevent default. Enter and
/// Space are left to the button's native click.
#[must_use]
pub fn trigger_keydown(key: &str, value: &str) -> Option<MenubarEvent> {
    match key {
        "ArrowDown" | "ArrowUp" => Some(MenubarEvent::Open(value.to_string())),
        "Home" => Some(MenubarEvent::Home),
        "End" => Some(MenubarEvent::End),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_active_trigger_is_a_tab_stop() {
        let input = MenubarInput { menus: vec![MenubarEntry::new("a"), MenubarEntry::new("b").disabled()], ..Default::default() };
        let state = MenubarState { active: Some("a".into()), open: Some("a".into()), controlled: false };
        let a = trigger_attrs(&state, &input, "a");
        assert_eq!((a.tabindex, a.data_state), ("0", "open"));
        let b = trigger_attrs(&state, &input, "b");
        assert_eq!((b.tabindex, b.data_state, b.disabled), ("-1", "closed", true));
    }

    #[test]
    fn keys_map_to_events() {
        assert_eq!(root_keydown("ArrowRight"), Some(MenubarEvent::Move(Direction::Next)));
        assert_eq!(root_keydown("Enter"), None);
        assert_eq!(trigger_keydown("ArrowDown", "a"), Some(MenubarEvent::Open("a".into())));
        assert_eq!(trigger_keydown("End", "a"), Some(MenubarEvent::End));
        assert_eq!(menu_root("ox-menubar-1", "file"), "ox-menubar-1-file");
    }
}
