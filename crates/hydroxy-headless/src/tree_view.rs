//! Tree view connector. Anatomy: tree, item, toggle, label. Every part carries
//! `data-scope="tree-view"` and its `data-part`. Rows render flat in display
//! order; `aria-level`, `aria-setsize` and `aria-posinset` carry the hierarchy.
//! Branch items carry `data-state="open" | "closed"`.

pub use hydroxy_machines::tree_view::*;

use crate::Orientation;
use crate::attrs::{data_state, nav_key, printable_char, roving};
use hydroxy_virtual::Nav;

/// The `data-scope` value on every tree view part.
pub const SCOPE: &str = "tree-view";

/// Attributes for one row (`<li role="treeitem">`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttrs {
    /// `id`, so the adapter can focus it.
    pub id: String,
    /// `aria-level`, from 1.
    pub aria_level: usize,
    /// `aria-setsize`.
    pub aria_setsize: usize,
    /// `aria-posinset`.
    pub aria_posinset: usize,
    /// `aria-expanded`. `None` for leaves.
    pub aria_expanded: Option<bool>,
    /// `aria-selected`, plus `data-selected`.
    pub selected: bool,
    /// `aria-disabled`, plus `data-disabled`.
    pub disabled: bool,
    /// `tabindex`: 0 for the roving tab stop, -1 otherwise.
    pub tabindex: i32,
    /// `data-state`: `"open"`, `"closed"`, or `None` for leaves.
    pub data_state: Option<&'static str>,
    /// `style`: `--ox-tree-level` for the indent.
    pub style: String,
}

/// The id of item `id`.
#[must_use]
pub fn item_id(root: &str, id: &str) -> String {
    format!("{root}-item-{id}")
}

/// Attributes for `row`.
#[must_use]
pub fn item_attrs(root: &str, state: &TreeViewState, row: &TreeRow, tab_stop: Option<&str>) -> ItemAttrs {
    let id = row.node.id.as_str();
    ItemAttrs {
        id: item_id(root, id),
        aria_level: row.level,
        aria_setsize: row.set_size,
        aria_posinset: row.pos_in_set,
        aria_expanded: row.node.is_branch().then_some(row.expanded),
        selected: state.selected.iter().any(|s| s == id),
        disabled: row.node.disabled,
        tabindex: roving(tab_stop == Some(id)),
        data_state: row.node.is_branch().then_some(data_state(row.expanded)),
        style: format!("--ox-tree-level: {}", row.level - 1),
    }
}

/// Map a `KeyboardEvent.key` on item `id` to a machine event. `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str, id: &str) -> Option<TreeViewEvent> {
    match nav_key(key, Some(Orientation::Vertical)) {
        Some(Nav::Next) => return Some(TreeViewEvent::Move(Direction::Next)),
        Some(Nav::Prev) => return Some(TreeViewEvent::Move(Direction::Prev)),
        Some(Nav::First) => return Some(TreeViewEvent::Home),
        Some(Nav::Last) => return Some(TreeViewEvent::End),
        _ => {}
    }
    match key {
        "ArrowRight" => Some(TreeViewEvent::ExpandOrEnter),
        "ArrowLeft" => Some(TreeViewEvent::CollapseOrExit),
        "Enter" | " " => Some(TreeViewEvent::Select(id.to_string())),
        "*" => Some(TreeViewEvent::ExpandSiblings),
        _ => printable_char(key).filter(|c| !c.is_whitespace()).map(TreeViewEvent::Typeahead),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_carry_hierarchy_and_one_tab_stop() {
        let nodes = vec![TreeNode::new("a", "A").children(vec![TreeNode::new("b", "B")]), TreeNode::new("c", "C")];
        let state = TreeViewState { expanded: vec!["a".into()], selected: vec!["b".into()], ..Default::default() };
        let rows = visible_rows(&nodes, &state.expanded);
        let stop = state.tab_stop(&rows);
        let a = item_attrs("t", &state, &rows[0], stop);
        assert_eq!((a.aria_expanded, a.data_state, a.tabindex), (Some(true), Some("open"), 0));
        let b = item_attrs("t", &state, &rows[1], stop);
        assert_eq!((b.aria_level, b.aria_expanded, b.selected, b.tabindex), (2, None, true, -1));
        assert_eq!(b.style, "--ox-tree-level: 1");
    }

    #[test]
    fn keys() {
        assert_eq!(keydown(" ", "x"), Some(TreeViewEvent::Select("x".into())));
        assert_eq!(keydown("*", "x"), Some(TreeViewEvent::ExpandSiblings));
        assert_eq!(keydown("q", "x"), Some(TreeViewEvent::Typeahead('q')));
        assert_eq!(keydown("Shift", "x"), None);
    }
}
