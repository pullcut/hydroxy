//! Dock layout connector. Anatomy: root, split, group, tablist, tab, close,
//! panel, instructions, live. Every part carries `data-scope="dock-layout"`
//! and its `data-part`. Each group is a tablist with roving focus.
//!
//! Pointer drags use native HTML5 drag and drop and mark the landing spot with
//! `data-drop-before` on a tab or `data-drop-end` on the group's tab strip.

pub use hydroxy_machines::dock_layout::*;

use crate::Orientation;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "dock-layout";

/// Read by screen readers on every tab, through `aria-describedby`.
pub const INSTRUCTIONS: &str = "Left and Right arrows switch tabs. Alt with Left or Right reorders the tab. Alt with Up or Down sends it to another group. Delete closes it.";

/// Attributes for a tab (`<div role="tab">`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabAttrs {
    /// `id`.
    pub id: String,
    /// `aria-selected`.
    pub aria_selected: bool,
    /// `aria-controls`: the panel's id.
    pub aria_controls: String,
    /// Roving tabindex: 0 for the shown tab of each group.
    pub tabindex: i32,
    /// `data-state`: `active` or `inactive`.
    pub data_state: &'static str,
    /// Emit `data-dragging` while this tab is dragged.
    pub dragging: bool,
    /// Emit `data-drop-before` when a drop would land just before this tab.
    pub drop_before: bool,
}

/// Attributes for a panel (`role="tabpanel"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelAttrs {
    /// `id`.
    pub id: String,
    /// `aria-labelledby`: the tab's id.
    pub aria_labelledby: String,
    /// `hidden` unless it is its group's shown panel.
    pub hidden: bool,
}

/// Panel `panel`'s tab id.
#[must_use]
pub fn tab_id(root: &str, panel: &str) -> String {
    format!("{root}-tab-{panel}")
}

/// Panel `panel`'s content id.
#[must_use]
pub fn panel_id(root: &str, panel: &str) -> String {
    format!("{root}-panel-{panel}")
}

/// The instructions' id.
#[must_use]
pub fn instructions_id(root: &str) -> String {
    format!("{root}-instructions")
}

/// The split axis as a resizable orientation.
#[must_use]
pub fn orientation(axis: SplitAxis) -> Orientation {
    match axis {
        SplitAxis::Horizontal => Orientation::Horizontal,
        SplitAxis::Vertical => Orientation::Vertical,
    }
}

/// Where a pointer drag would land: (group, tab it goes before, or `None` for the end).
#[must_use]
pub fn drop_target(state: &DockState) -> Option<(&str, Option<&str>)> {
    state.drag.as_ref().map(|d| (d.group.as_str(), d.before.as_deref()))
}

/// Whether a drop would land at the end of `group`.
#[must_use]
pub fn drop_at_end(state: &DockState, group: &str) -> bool {
    drop_target(state).is_some_and(|(g, before)| g == group && before.is_none())
}

/// Attributes for the tab of `panel` in `group`.
#[must_use]
pub fn tab_attrs(root: &str, state: &DockState, group: &DockGroup, panel: &str) -> TabAttrs {
    let shown = group.shown() == Some(panel);
    let dragged = state.drag.as_ref().is_some_and(|d| d.panel == panel);
    TabAttrs {
        id: tab_id(root, panel),
        aria_selected: shown,
        aria_controls: panel_id(root, panel),
        tabindex: if shown { 0 } else { -1 },
        data_state: if shown { "active" } else { "inactive" },
        dragging: dragged,
        drop_before: !dragged && drop_target(state).is_some_and(|(g, before)| g == group.id && before == Some(panel)),
    }
}

/// Attributes for the panel of `panel` in `group`.
#[must_use]
pub fn panel_attrs(root: &str, group: &DockGroup, panel: &str) -> PanelAttrs {
    PanelAttrs { id: panel_id(root, panel), aria_labelledby: tab_id(root, panel), hidden: group.shown() != Some(panel) }
}

/// The event for a key on the tab of `panel`, if the key does anything. `Some` means: prevent default.
#[must_use]
pub fn tab_keydown(key: &str, alt: bool, panel: &str, state: &DockState, input: &DockInput) -> Option<DockEvent> {
    let group = state.tree.group_of(panel)?;
    let at = group.panels.iter().position(|p| p == panel)?;
    let owned = || panel.to_string();
    let neighbour = |to: usize| group.panels.get(to).cloned().map(DockEvent::Activate);
    match (key, alt) {
        ("ArrowLeft", true) => Some(DockEvent::Reorder { panel: owned(), delta: -1 }),
        ("ArrowRight", true) => Some(DockEvent::Reorder { panel: owned(), delta: 1 }),
        ("ArrowUp", true) => Some(DockEvent::SendToGroup { panel: owned(), delta: -1 }),
        ("ArrowDown", true) => Some(DockEvent::SendToGroup { panel: owned(), delta: 1 }),
        ("ArrowLeft", false) => neighbour(at.checked_sub(1).unwrap_or(group.panels.len() - 1)),
        ("ArrowRight", false) => neighbour(if at + 1 < group.panels.len() { at + 1 } else { 0 }),
        ("Home", false) => neighbour(0),
        ("End", false) => neighbour(group.panels.len() - 1),
        ("Delete" | "Backspace", false) if !input.locked.iter().any(|l| l == panel) => Some(DockEvent::Close(owned())),
        _ => None,
    }
}

/// Default English wording for an announcement. `title` is the panel's visible name.
#[must_use]
pub fn announcement_text(a: &Announcement, title: &str) -> String {
    match a {
        Announcement::Moved { group, groups, position, count, .. } => {
            format!("{title} moved to position {position} of {count} in group {group} of {groups}.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> (DockState, DockInput) {
        let tree = DockNode::split(
            "root",
            SplitAxis::Horizontal,
            vec![DockNode::tabs("l", &["a", "b", "c"]), DockNode::tabs("r", &["d"])],
        );
        let input = DockInput { default_tree: tree.clone(), locked: vec!["d".into()], ..DockInput::default() };
        (DockState { tree, drag: None, controlled: false }, input)
    }

    #[test]
    fn arrows_wrap_and_alt_arrows_move() {
        let (s, i) = state();
        assert_eq!(tab_keydown("ArrowLeft", false, "a", &s, &i), Some(DockEvent::Activate("c".into())));
        assert_eq!(tab_keydown("ArrowRight", false, "c", &s, &i), Some(DockEvent::Activate("a".into())));
        assert_eq!(tab_keydown("End", false, "a", &s, &i), Some(DockEvent::Activate("c".into())));
        assert_eq!(tab_keydown("ArrowDown", true, "a", &s, &i), Some(DockEvent::SendToGroup { panel: "a".into(), delta: 1 }));
        assert_eq!(tab_keydown("Delete", false, "d", &s, &i), None, "locked");
        assert_eq!(tab_keydown("Delete", false, "a", &s, &i), Some(DockEvent::Close("a".into())));
    }

    #[test]
    fn the_shown_tab_is_the_tab_stop_and_drops_are_marked() {
        let (mut s, _) = state();
        let g = s.tree.group_of("a").unwrap().clone();
        assert_eq!(tab_attrs("x", &s, &g, "a").tabindex, 0);
        assert_eq!(tab_attrs("x", &s, &g, "b").tabindex, -1);
        assert!(panel_attrs("x", &g, "b").hidden && !panel_attrs("x", &g, "a").hidden);
        s.drag = Some(Drag { panel: "d".into(), group: "l".into(), before: Some("b".into()) });
        assert!(tab_attrs("x", &s, &g, "b").drop_before);
        assert!(!drop_at_end(&s, "l"));
        s.drag.as_mut().unwrap().before = None;
        assert!(drop_at_end(&s, "l"));
    }
}
