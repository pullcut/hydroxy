//! Spreadsheet grid connector. Anatomy: root, formula-bar, name-box, formula,
//! viewport, header-row, corner, column-header, resize-handle, body,
//! selection, row, row-header, cell, editor, status. Every part carries
//! `data-scope="spreadsheet-grid"` and its `data-part`.
//!
//! The viewport is `role="grid"` and holds DOM focus; the active cell is
//! announced through `aria-activedescendant`, so cells can be windowed in and
//! out without losing focus. Indexes on the wire are 1-based and count the
//! header row and the row-number column, as ARIA wants.

use std::fmt::Write as _;

use hydroxy_machines::Modifiers;
pub use hydroxy_machines::shortcut_recorder::KeyPress;
pub use hydroxy_machines::spreadsheet_grid::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "spreadsheet-grid";

/// Pixels a resize handle's arrow key adds or removes (Shift: four times as much).
pub const RESIZE_STEP: f64 = 8.0;

/// The viewport's id, for scrolling and focus.
#[must_use]
pub fn viewport_id(root: &str) -> String {
    format!("{root}-viewport")
}

/// The id of cell `(row, col)`.
#[must_use]
pub fn cell_id(root: &str, (row, col): Cell) -> String {
    format!("{root}-r{row}-c{col}")
}

/// The editor input's id.
#[must_use]
pub fn editor_id(root: &str) -> String {
    format!("{root}-editor")
}

/// `aria-rowcount`: the rows plus the header row.
#[must_use]
pub fn aria_rowcount(input: &SpreadsheetGridInput) -> usize {
    input.rows + 1
}

/// `aria-colcount`: the columns plus the row-number column.
#[must_use]
pub fn aria_colcount(input: &SpreadsheetGridInput) -> usize {
    input.cols + 1
}

/// `aria-rowindex` of body row `row` (0-based).
#[must_use]
pub fn aria_rowindex(row: usize) -> usize {
    row + 2
}

/// `aria-colindex` of column `col` (0-based).
#[must_use]
pub fn aria_colindex(col: usize) -> usize {
    col + 2
}

/// The viewport's inline style: its height.
#[must_use]
pub fn viewport_style(input: &SpreadsheetGridInput) -> String {
    format!("height: {}px;", input.height)
}

/// `grid-template-columns` shared by the header row and every body row.
#[must_use]
pub fn template_columns(state: &SpreadsheetGridState) -> String {
    let mut s = format!("grid-template-columns: {ROW_HEADER_WIDTH}px");
    for w in &state.widths {
        let _ = write!(s, " {w}px");
    }
    let _ = write!(s, "; width: {}px;", state.total_width());
    s
}

/// The body's inline style: tall enough for every row, so the scrollbar is true.
#[must_use]
pub fn body_style(state: &SpreadsheetGridState, input: &SpreadsheetGridInput) -> String {
    format!("height: {}px; width: {}px;", row_top(input.rows), state.total_width())
}

/// Top of body row `row`, in pixels; also the height of `row` rows.
#[allow(clippy::cast_precision_loss, reason = "row counts are far below 2^52")]
#[must_use]
pub fn row_top(row: usize) -> f64 {
    row as f64 * ROW_HEIGHT
}

/// Rows to render: the window around the viewport, plus the active row when
/// it has been scrolled away, so `aria-activedescendant` always resolves.
#[must_use]
pub fn rendered_rows(state: &SpreadsheetGridState, input: &SpreadsheetGridInput) -> Vec<usize> {
    let window = state.visible_rows(input);
    let active = state.active.0;
    let mut rows: Vec<usize> = window.clone().collect();
    if active < input.rows && !window.contains(&active) {
        let at = rows.partition_point(|r| *r < active);
        rows.insert(at, active);
    }
    rows
}

/// A body row's inline style: its offset and the column template from
/// [`template_columns`]. Plain `top`, not a transform, so rows make no stacking
/// context and the sticky row numbers stay above the selection outline.
#[must_use]
pub fn row_style(row: usize, template: &str) -> String {
    format!("top: {}px; {template}", row_top(row))
}

/// The selection outline's inline style, in body coordinates.
#[must_use]
pub fn selection_style(state: &SpreadsheetGridState) -> String {
    let r = state.selection();
    let left = state.column_left(r.left());
    let width = state.column_left(r.right() + 1) - left;
    format!(
        "top: {}px; left: {left}px; width: {width}px; height: {}px;",
        row_top(r.top()),
        row_top(r.bottom() - r.top() + 1)
    )
}

/// The name box: `B3`, or `A1:C4` for a range.
#[must_use]
pub fn name_box(state: &SpreadsheetGridState) -> String {
    state.selection().name()
}

/// The formula bar: the editor text while editing, else the active cell's raw text.
#[must_use]
pub fn formula_text(state: &SpreadsheetGridState) -> String {
    state.editing.clone().unwrap_or_else(|| state.raw(state.active).to_string())
}

/// Attributes for one body cell (`role="gridcell"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellAttrs {
    /// `id`, the target of `aria-activedescendant`.
    pub id: String,
    /// `aria-colindex`.
    pub aria_colindex: usize,
    /// `aria-selected`.
    pub selected: bool,
    /// The active cell: `data-active`.
    pub active: bool,
    /// `data-kind`: `empty`, `number`, `text` or `error`. Numbers align right.
    pub kind: &'static str,
    /// The text shown.
    pub text: String,
}

/// Attributes for cell `cell`.
#[must_use]
pub fn cell_attrs(root: &str, state: &SpreadsheetGridState, cell: Cell) -> CellAttrs {
    CellAttrs {
        id: cell_id(root, cell),
        aria_colindex: aria_colindex(cell.1),
        selected: state.selection().contains(cell),
        active: state.active == cell,
        kind: state.value(cell).kind(),
        text: state.display(cell),
    }
}

/// Attributes for column `col`'s resize handle (`role="separator"`).
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeAttrs {
    /// `aria-label`.
    pub label: String,
    /// `aria-valuenow`: the width in pixels.
    pub valuenow: f64,
    /// `aria-valuemin`.
    pub valuemin: f64,
    /// `aria-valuemax`.
    pub valuemax: f64,
    /// `data-dragging`.
    pub dragging: bool,
}

/// Attributes for column `col`'s resize handle.
#[must_use]
pub fn resize_attrs(state: &SpreadsheetGridState, col: usize) -> ResizeAttrs {
    ResizeAttrs {
        label: format!("Resize column {}", column_name(col)),
        valuenow: state.widths.get(col).copied().unwrap_or_default().round(),
        valuemin: MIN_COL_WIDTH,
        valuemax: MAX_COL_WIDTH,
        dragging: state.resizing.is_some_and(|r| r.col == col),
    }
}

/// The event for a key on a resize handle, if the key does anything.
#[must_use]
pub fn resize_keydown(key: &str, mods: Modifiers, col: usize) -> Option<SpreadsheetGridEvent> {
    let step = if mods.shift { RESIZE_STEP * 4.0 } else { RESIZE_STEP };
    let delta = match key {
        "ArrowLeft" => -step,
        "ArrowRight" => step,
        "Home" => -MAX_COL_WIDTH,
        "End" => MAX_COL_WIDTH,
        _ => return None,
    };
    Some(SpreadsheetGridEvent::ResizeBy { col, delta })
}

/// The event for a key on the grid (not in the editor). `Some` means: prevent
/// default. Ctrl+C, X and V return `None` so the clipboard events fire. Ctrl and Cmd
/// both count as the command key.
#[must_use]
pub fn grid_keydown(press: &KeyPress) -> Option<SpreadsheetGridEvent> {
    let key = press.key.as_str();
    let ctrl = press.ctrl || press.meta;
    let nav = match (key, ctrl) {
        ("ArrowUp", false) => Nav::Up,
        ("ArrowDown", false) => Nav::Down,
        ("ArrowLeft", false) => Nav::Left,
        ("ArrowRight", false) => Nav::Right,
        ("ArrowUp", true) => Nav::Top,
        ("ArrowDown", true) => Nav::Bottom,
        ("ArrowLeft", true) | ("Home", false) => Nav::RowStart,
        ("ArrowRight", true) | ("End", false) => Nav::RowEnd,
        ("Home", true) => Nav::SheetStart,
        ("End", true) => Nav::SheetEnd,
        ("PageUp", _) => Nav::PageUp,
        ("PageDown", _) => Nav::PageDown,
        ("Enter" | "F2", _) => return Some(SpreadsheetGridEvent::StartEdit(None)),
        ("Delete" | "Backspace", _) => return Some(SpreadsheetGridEvent::Clear),
        ("Escape", _) => return Some(SpreadsheetGridEvent::Collapse),
        ("a" | "A", true) => return Some(SpreadsheetGridEvent::SelectAll),
        (k, false) if !press.alt && crate::attrs::printable_char(k).is_some() => {
            return Some(SpreadsheetGridEvent::StartEdit(Some(k.to_string())));
        }
        _ => return None,
    };
    Some(SpreadsheetGridEvent::Move { nav, extend: press.shift })
}

/// The event for a key in the editor, if the key does anything. `Some` means:
/// prevent default and stop propagation.
#[must_use]
pub fn editor_keydown(key: &str, mods: Modifiers) -> Option<SpreadsheetGridEvent> {
    let after = match (key, mods.shift) {
        ("Enter", false) => After::Down,
        ("Enter", true) => After::Up,
        ("Tab", false) => After::Right,
        ("Tab", true) => After::Left,
        ("Escape", _) => return Some(SpreadsheetGridEvent::Cancel),
        _ => return None,
    };
    Some(SpreadsheetGridEvent::Commit(after))
}

/// Whether a key press is Ctrl/Cmd+C (copy) or Ctrl/Cmd+X (cut), for adapters
/// that write the clipboard themselves. Returns `Some(true)` for cut.
#[must_use]
pub fn copy_shortcut(press: &KeyPress) -> Option<bool> {
    let command = (press.ctrl || press.meta) && !press.alt;
    match press.key.as_str() {
        "c" | "C" if command => Some(false),
        "x" | "X" if command => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use hydroxy_statechart::Machine;

    use super::*;

    fn state() -> (SpreadsheetGridState, SpreadsheetGridInput) {
        let cells: BTreeMap<Cell, String> =
            [((0, 0), "4".to_string()), ((0, 1), "=A1/0".to_string())].into_iter().collect();
        let input = SpreadsheetGridInput {
            rows: 50,
            cols: 3,
            default_cells: Arc::new(cells),
            ..SpreadsheetGridInput::default()
        };
        (SpreadsheetGridMachine::initial(&input), input)
    }

    #[test]
    fn cells_report_selection_kind_and_text() {
        let (s, _) = state();
        let a = cell_attrs("g", &s, (0, 0));
        assert_eq!(
            (a.id.as_str(), a.aria_colindex, a.selected, a.active, a.kind),
            ("g-r0-c0", 2, true, true, "number")
        );
        let b = cell_attrs("g", &s, (0, 1));
        assert_eq!((b.kind, b.text.as_str(), b.selected), ("error", "#DIV/0!", false));
        assert_eq!(formula_text(&SpreadsheetGridState { active: (0, 1), ..s.clone() }), "=A1/0");
        assert_eq!(name_box(&s), "A1");
    }

    #[test]
    fn layout_styles_follow_widths_and_rows() {
        let (s, i) = state();
        assert_eq!(template_columns(&s), "grid-template-columns: 52px 112px 112px 112px; width: 388px;");
        assert_eq!(body_style(&s, &i), "height: 1400px; width: 388px;");
        let s = SpreadsheetGridState { active: (2, 1), extent: (3, 2), ..s };
        assert_eq!(selection_style(&s), "top: 56px; left: 164px; width: 224px; height: 56px;");
        assert_eq!((aria_rowcount(&i), aria_colcount(&i), aria_rowindex(0)), (51, 4, 2));
        let away = SpreadsheetGridState { active: (40, 0), extent: (40, 0), ..s };
        let rows = rendered_rows(&away, &i);
        assert_eq!(rows.last(), Some(&40), "the active row stays rendered");
        assert_eq!(rows.len(), 19 + 1);
    }

    #[test]
    fn keys_map_to_events() {
        let press =
            |key: &str, shift: bool, ctrl: bool| KeyPress { key: key.into(), shift, ctrl, ..KeyPress::default() };
        let none = |key: &str| press(key, false, false);
        let shift = |key: &str| press(key, true, false);
        let ctrl = |key: &str| press(key, false, true);
        assert_eq!(
            grid_keydown(&shift("ArrowDown")),
            Some(SpreadsheetGridEvent::Move { nav: Nav::Down, extend: true })
        );
        assert_eq!(grid_keydown(&ctrl("End")), Some(SpreadsheetGridEvent::Move { nav: Nav::SheetEnd, extend: false }));
        assert_eq!(grid_keydown(&none("=")), Some(SpreadsheetGridEvent::StartEdit(Some("=".into()))));
        assert_eq!(grid_keydown(&none("F2")), Some(SpreadsheetGridEvent::StartEdit(None)));
        assert_eq!(grid_keydown(&ctrl("a")), Some(SpreadsheetGridEvent::SelectAll));
        assert_eq!(grid_keydown(&ctrl("c")), None, "copy is left to the clipboard event");
        assert_eq!(grid_keydown(&none("Tab")), None, "Tab leaves the grid");
        assert_eq!(editor_keydown("Tab", Modifiers::SHIFT), Some(SpreadsheetGridEvent::Commit(After::Left)));
        assert_eq!(editor_keydown("a", Modifiers::NONE), None);
        assert_eq!(copy_shortcut(&ctrl("x")), Some(true));
        assert_eq!(
            resize_keydown("ArrowRight", Modifiers::SHIFT, 2),
            Some(SpreadsheetGridEvent::ResizeBy { col: 2, delta: 32.0 })
        );
    }
}
