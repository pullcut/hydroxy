//! Grid connector. Anatomy: root, header, header-row, header-cell, body, row,
//! cell, editor, live. Every part carries `data-scope="grid"` and its
//! `data-part`. The root is `role="grid"`; exactly one cell has `tabindex="0"`.
//!
//! Indexes are 1-based on the wire and count the header row, as ARIA wants:
//! the first body row is `aria-rowindex="2"`.

pub use hydroxy_machines::grid::*;

use crate::attrs::roving;

/// The `data-scope` value on every grid part.
pub const SCOPE: &str = "grid";

/// Attributes for a body cell (`role="gridcell"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellAttrs {
    /// `id`, so the adapter can focus it.
    pub id: String,
    /// Roving tabindex: 0 for one cell, -1 for the rest.
    pub tabindex: i32,
    /// `aria-colindex`.
    pub aria_colindex: usize,
    /// `aria-readonly`: set on cells that cannot be edited.
    pub readonly: bool,
    /// The cell holds its editor.
    pub editing: bool,
}

/// Cell `(row, col)`'s id.
#[must_use]
pub fn cell_id(root: &str, row: usize, col: usize) -> String {
    format!("{root}-cell-{row}-{col}")
}

/// The editor input's id.
#[must_use]
pub fn editor_id(root: &str, row: usize, col: usize) -> String {
    format!("{root}-editor-{row}-{col}")
}

/// `aria-rowcount`: the body rows plus the header.
#[must_use]
pub fn aria_rowcount(state: &GridState) -> usize {
    state.rows.value.len() + 1
}

/// `aria-rowindex` for body row `row` (0-based).
#[must_use]
pub fn aria_rowindex(row: usize) -> usize {
    row + 2
}

/// Attributes for cell `(row, col)`.
#[must_use]
pub fn cell_attrs(root: &str, state: &GridState, input: &GridInput, row: usize, col: usize) -> CellAttrs {
    CellAttrs {
        id: cell_id(root, row, col),
        tabindex: roving(state.tabbable() == (row, col)),
        aria_colindex: col + 1,
        readonly: !input.columns.get(col).is_some_and(|c| c.editable),
        editing: state.editing.as_ref().is_some_and(|e| (e.row, e.col) == (row, col)),
    }
}

/// The event for a key on a cell, if the key does anything. `Some` means: prevent default.
#[must_use]
pub fn cell_keydown(key: &str, ctrl: bool) -> Option<GridEvent> {
    let nav = match (key, ctrl) {
        ("ArrowUp", _) => Nav::Up,
        ("ArrowDown", _) => Nav::Down,
        ("ArrowLeft", _) => Nav::Left,
        ("ArrowRight", _) => Nav::Right,
        ("Home", true) => Nav::GridStart,
        ("Home", false) => Nav::RowStart,
        ("End", true) => Nav::GridEnd,
        ("End", false) => Nav::RowEnd,
        ("PageUp", _) => Nav::PageUp,
        ("PageDown", _) => Nav::PageDown,
        ("Enter" | "F2", _) => return Some(GridEvent::StartEdit),
        _ => return None,
    };
    Some(GridEvent::Move(nav))
}

/// The event for a key in the editor, if the key does anything.
#[must_use]
pub fn editor_keydown(key: &str) -> Option<GridEvent> {
    match key {
        "Enter" => Some(GridEvent::Commit),
        "Escape" => Some(GridEvent::Cancel),
        _ => None,
    }
}

/// The CSS `grid-template-columns` shared by the header row and every row.
#[must_use]
pub fn template_columns(input: &GridInput) -> String {
    format!("grid-template-columns: repeat({}, minmax(0, 1fr));", input.columns.len().max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> GridInput {
        GridInput {
            columns: vec![GridColumn::new("a", "A"), GridColumn::new("b", "B").editable()],
            ..GridInput::default()
        }
    }

    #[test]
    fn one_cell_is_tabbable() {
        let s = GridState { focus: Some((1, 1)), ..GridState::default() };
        let tab: Vec<i32> =
            [(0, 0), (1, 1), (1, 0)].iter().map(|&(r, c)| cell_attrs("g", &s, &input(), r, c).tabindex).collect();
        assert_eq!(tab, [-1, 0, -1]);
        assert_eq!(
            cell_attrs("g", &GridState::default(), &input(), 0, 0).tabindex,
            0,
            "top left until something is focused"
        );
        assert!(cell_attrs("g", &s, &input(), 0, 0).readonly);
        assert!(!cell_attrs("g", &s, &input(), 0, 1).readonly);
    }

    #[test]
    fn keys_map_to_two_dimensional_moves() {
        assert_eq!(cell_keydown("Home", true), Some(GridEvent::Move(Nav::GridStart)));
        assert_eq!(cell_keydown("Home", false), Some(GridEvent::Move(Nav::RowStart)));
        assert_eq!(cell_keydown("F2", false), Some(GridEvent::StartEdit));
        assert_eq!(cell_keydown("a", false), None);
        assert_eq!(editor_keydown("Escape"), Some(GridEvent::Cancel));
        assert_eq!((aria_rowindex(0), aria_rowcount(&GridState::default())), (2, 1));
    }
}
