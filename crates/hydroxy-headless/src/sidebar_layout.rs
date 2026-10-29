//! Sidebar layout connector. Anatomy: root, sidebar, toggle, mobile-toggle,
//! header, main. Every part carries `data-scope="sidebar-layout"` and its
//! `data-part`; root and sidebar carry `data-state="expanded|rail|collapsed"`.
//!
//! The open flag reuses the collapsible machine. On narrow screens the sidebar
//! is a native `popover`, opened by the mobile toggle with `popovertarget`.

use hydroxy_machines::Modifiers;
pub use hydroxy_machines::collapsible::{
    CollapsibleCommand, CollapsibleEvent, CollapsibleInput, CollapsibleMachine, CollapsibleState,
};

/// `data-scope` for every part.
pub const SCOPE: &str = "sidebar-layout";

/// What a closed sidebar turns into on wide screens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CollapseMode {
    /// A narrow icon rail. Labels stay readable by assistive tech.
    #[default]
    Rail,
    /// Hidden entirely.
    Hidden,
}

/// `data-state` for the root and sidebar.
#[must_use]
pub fn data_state(open: bool, mode: CollapseMode) -> &'static str {
    match (open, mode) {
        (true, _) => "expanded",
        (false, CollapseMode::Rail) => "rail",
        (false, CollapseMode::Hidden) => "collapsed",
    }
}

/// The sidebar element id, the toggles' `aria-controls` and `popovertarget`.
#[must_use]
pub fn sidebar_id(root: &str) -> String {
    format!("{root}-sidebar")
}

/// Ctrl+B or Cmd+B toggles the sidebar.
#[must_use]
pub fn is_toggle_shortcut(key: &str, mods: Modifiers) -> bool {
    mods.command() && key.eq_ignore_ascii_case("b")
}

/// The toggle's accessible name. Constant: `aria-expanded` carries the state.
pub const TOGGLE_LABEL: &str = "Toggle sidebar";

/// The mobile toggle's accessible name.
pub const MOBILE_TOGGLE_LABEL: &str = "Open navigation";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_and_shortcut() {
        assert_eq!(data_state(true, CollapseMode::Hidden), "expanded");
        assert_eq!(data_state(false, CollapseMode::Rail), "rail");
        assert_eq!(data_state(false, CollapseMode::Hidden), "collapsed");
        assert!(is_toggle_shortcut("B", Modifiers::META));
        assert!(!is_toggle_shortcut("b", Modifiers::NONE));
    }
}
