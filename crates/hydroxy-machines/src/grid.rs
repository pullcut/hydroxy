//! Data grid: the ARIA `grid` pattern. One cell is in the tab order and arrow
//! keys move between cells in two dimensions. Enter or F2 edits the focused
//! cell; Enter saves, Escape discards. Unlike a data table, a grid is for
//! working inside the data, so it has no sorting or selection of its own.

use hydroxy_statechart::{Machine, Transition};

/// One column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridColumn {
    /// Unique key, reported with edits.
    pub key: String,
    /// Header text.
    pub header: String,
    /// Cells in this column can be edited.
    pub editable: bool,
}

impl GridColumn {
    /// A read-only column.
    #[must_use]
    pub fn new(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self { key: key.into(), header: header.into(), editable: false }
    }

    /// Make the column editable.
    #[must_use]
    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }
}

/// One row: an id and a text per column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRow {
    /// Unique key, reported with edits.
    pub id: String,
    /// Cell text, in column order.
    pub cells: Vec<String>,
}

impl GridRow {
    /// A row.
    #[must_use]
    pub fn new(id: impl Into<String>, cells: &[&str]) -> Self {
        Self { id: id.into(), cells: cells.iter().map(|c| (*c).to_string()).collect() }
    }
}

/// A saved edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridEdit {
    /// The row's id.
    pub row: String,
    /// The column's key.
    pub column: String,
    /// The new text.
    pub value: String,
    /// The text before.
    pub previous: String,
}

/// Props for [`GridMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridInput {
    /// Columns, left to right.
    pub columns: Vec<GridColumn>,
    /// Controlled rows. `Some` at start fixes controlled mode.
    pub rows: Option<Vec<GridRow>>,
    /// Uncontrolled initial rows.
    pub default_rows: Vec<GridRow>,
    /// Rows a page jump skips.
    pub page_size: usize,
}

impl Default for GridInput {
    fn default() -> Self {
        Self { columns: Vec::new(), rows: None, default_rows: Vec::new(), page_size: 10 }
    }
}

/// A cell being edited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Editing {
    /// Row index.
    pub row: usize,
    /// Column index.
    pub col: usize,
    /// The text in the editor.
    pub draft: String,
}

/// What [`GridMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridState {
    /// The committed rows.
    pub rows: Vec<GridRow>,
    /// The focused cell, as (row, column).
    pub focus: Option<(usize, usize)>,
    /// The cell being edited.
    pub editing: Option<Editing>,
    /// Fixed at start: the owner holds the rows.
    pub controlled: bool,
}

impl GridState {
    /// The cell in the tab order: the focused one, else the top left.
    #[must_use]
    pub fn tabbable(&self) -> (usize, usize) {
        self.focus.unwrap_or((0, 0))
    }
}

/// Where a navigation key goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    /// Up one row.
    Up,
    /// Down one row.
    Down,
    /// Left one column.
    Left,
    /// Right one column.
    Right,
    /// First column of the row.
    RowStart,
    /// Last column of the row.
    RowEnd,
    /// First cell of the grid.
    GridStart,
    /// Last cell of the grid.
    GridEnd,
    /// Up a page of rows.
    PageUp,
    /// Down a page of rows.
    PageDown,
}

/// Inputs to [`GridMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridEvent {
    /// A cell took DOM focus (pointer or Tab).
    Focus {
        /// Row index.
        row: usize,
        /// Column index.
        col: usize,
    },
    /// A navigation key.
    Move(Nav),
    /// Enter or F2 on the focused cell.
    StartEdit,
    /// Double click on a cell.
    StartEditAt {
        /// Row index.
        row: usize,
        /// Column index.
        col: usize,
    },
    /// The editor's text changed.
    EditChange(String),
    /// Enter in the editor: save and return focus to the cell.
    Commit,
    /// The editor lost focus: save, but leave focus where the user put it.
    Blur,
    /// Escape in the editor: discard and return focus to the cell.
    Cancel,
}

/// Side effects from [`GridMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridCommand {
    /// Move DOM focus to this cell.
    FocusCell {
        /// Row index.
        row: usize,
        /// Column index.
        col: usize,
    },
    /// Move DOM focus into this cell's editor once it exists.
    FocusEditor {
        /// Row index.
        row: usize,
        /// Column index.
        col: usize,
    },
    /// A cell was edited (or should be, when controlled).
    Edit(GridEdit),
}

/// Focus and editing state for a grid.
#[derive(Debug, Clone, Copy)]
pub struct GridMachine;

type Step = Transition<GridState, GridCommand>;

fn start_edit(state: &GridState, input: &GridInput, row: usize, col: usize) -> Step {
    let editable = input.columns.get(col).is_some_and(|c| c.editable);
    let Some(text) = state.rows.get(row).and_then(|r| r.cells.get(col)) else { return Transition::stay() };
    if !editable {
        return Transition::stay();
    }
    let mut next = state.clone();
    next.focus = Some((row, col));
    next.editing = Some(Editing { row, col, draft: text.clone() });
    Transition::with(next, vec![GridCommand::FocusEditor { row, col }])
}

fn finish(state: &GridState, input: &GridInput, save: bool, refocus: bool) -> Step {
    let Some(e) = &state.editing else { return Transition::stay() };
    let mut next = state.clone();
    next.editing = None;
    let mut commands = Vec::new();
    let previous = state.rows.get(e.row).and_then(|r| r.cells.get(e.col)).cloned().unwrap_or_default();
    if save && e.draft != previous {
        if let (Some(row), Some(column)) = (state.rows.get(e.row), input.columns.get(e.col)) {
            commands.push(GridCommand::Edit(GridEdit {
                row: row.id.clone(),
                column: column.key.clone(),
                value: e.draft.clone(),
                previous,
            }));
        }
        if !state.controlled
            && let Some(cell) = next.rows.get_mut(e.row).and_then(|r| r.cells.get_mut(e.col))
        {
            cell.clone_from(&e.draft);
        }
    }
    if refocus {
        commands.push(GridCommand::FocusCell { row: e.row, col: e.col });
    }
    Transition::with(next, commands)
}

impl Machine for GridMachine {
    type Input = GridInput;
    type State = GridState;
    type Event = GridEvent;
    type Command = GridCommand;

    fn initial(input: &GridInput) -> GridState {
        GridState {
            rows: input.rows.clone().unwrap_or_else(|| input.default_rows.clone()),
            focus: None,
            editing: None,
            controlled: input.rows.is_some(),
        }
    }

    fn transition(state: &GridState, event: GridEvent, input: &GridInput) -> Step {
        let stay = || Transition::stay();
        let (rows, cols) = (state.rows.len(), input.columns.len());
        match event {
            GridEvent::Focus { row, col } => {
                if state.editing.is_some() || row >= rows || col >= cols || state.focus == Some((row, col)) {
                    return stay();
                }
                let mut next = state.clone();
                next.focus = Some((row, col));
                Transition::to(next)
            }
            GridEvent::Move(nav) if state.editing.is_none() && rows > 0 && cols > 0 => {
                let (r, c) = state.tabbable();
                let page = input.page_size.max(1);
                let (nr, nc) = match nav {
                    Nav::Up => (r.saturating_sub(1), c),
                    Nav::Down => ((r + 1).min(rows - 1), c),
                    Nav::Left => (r, c.saturating_sub(1)),
                    Nav::Right => (r, (c + 1).min(cols - 1)),
                    Nav::RowStart => (r, 0),
                    Nav::RowEnd => (r, cols - 1),
                    Nav::GridStart => (0, 0),
                    Nav::GridEnd => (rows - 1, cols - 1),
                    Nav::PageUp => (r.saturating_sub(page), c),
                    Nav::PageDown => ((r + page).min(rows - 1), c),
                };
                if (r, c) == (nr, nc) {
                    return stay();
                }
                let mut next = state.clone();
                next.focus = Some((nr, nc));
                Transition::with(next, vec![GridCommand::FocusCell { row: nr, col: nc }])
            }
            GridEvent::StartEdit if state.editing.is_none() => {
                let (r, c) = state.tabbable();
                start_edit(state, input, r, c)
            }
            GridEvent::StartEditAt { row, col } if state.editing.is_none() => start_edit(state, input, row, col),
            GridEvent::EditChange(draft) => {
                let Some(e) = &state.editing else { return stay() };
                let mut next = state.clone();
                next.editing = Some(Editing { draft, ..e.clone() });
                Transition::to(next)
            }
            GridEvent::Commit => finish(state, input, true, true),
            GridEvent::Blur => finish(state, input, true, false),
            GridEvent::Cancel => finish(state, input, false, true),
            _ => stay(),
        }
    }

    fn sync(state: &GridState, input: &GridInput) -> Option<GridState> {
        let mut next = state.clone();
        if state.controlled {
            next.rows = input.rows.clone().unwrap_or_default();
        }
        let in_range = |(r, c): (usize, usize)| r < next.rows.len() && c < input.columns.len();
        if next.focus.is_some_and(|f| !in_range(f)) {
            next.focus = None;
        }
        if next.editing.as_ref().is_some_and(|e| !in_range((e.row, e.col))) {
            next.editing = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> GridInput {
        GridInput {
            columns: vec![GridColumn::new("name", "Name"), GridColumn::new("qty", "Qty").editable(), GridColumn::new("note", "Note")],
            default_rows: (0..25).map(|i| GridRow::new(format!("r{i}"), &["n", "1", "x"])).collect(),
            ..GridInput::default()
        }
    }

    fn run(state: &GridState, event: GridEvent, input: &GridInput) -> hydroxy_statechart::Step<GridState, GridCommand> {
        GridMachine::transition(state, event, input).resolve(state)
    }

    fn at(input: &GridInput, row: usize, col: usize) -> GridState {
        GridState { focus: Some((row, col)), ..GridMachine::initial(input) }
    }

    #[test]
    fn arrows_clamp_at_the_edges() {
        let i = input();
        let s = GridMachine::initial(&i);
        assert!(run(&s, GridEvent::Move(Nav::Up), &i).commands.is_empty(), "top left stays");
        let s = run(&s, GridEvent::Move(Nav::Right), &i).state;
        assert_eq!(s.focus, Some((0, 1)));
        let s = run(&s, GridEvent::Move(Nav::Down), &i).state;
        assert_eq!(s.focus, Some((1, 1)));
        assert_eq!(run(&at(&i, 0, 2), GridEvent::Move(Nav::Right), &i).state.focus, Some((0, 2)));
    }

    #[test]
    fn home_end_page_and_corners() {
        let i = input();
        let s = at(&i, 12, 1);
        assert_eq!(run(&s, GridEvent::Move(Nav::RowStart), &i).state.focus, Some((12, 0)));
        assert_eq!(run(&s, GridEvent::Move(Nav::RowEnd), &i).state.focus, Some((12, 2)));
        assert_eq!(run(&s, GridEvent::Move(Nav::GridStart), &i).state.focus, Some((0, 0)));
        assert_eq!(run(&s, GridEvent::Move(Nav::GridEnd), &i).state.focus, Some((24, 2)));
        assert_eq!(run(&s, GridEvent::Move(Nav::PageDown), &i).state.focus, Some((22, 1)));
        assert_eq!(run(&run(&s, GridEvent::Move(Nav::PageDown), &i).state, GridEvent::Move(Nav::PageDown), &i).state.focus, Some((24, 1)));
        assert_eq!(run(&s, GridEvent::Move(Nav::PageUp), &i).state.focus, Some((2, 1)));
    }

    #[test]
    fn only_editable_cells_edit() {
        let i = input();
        assert!(run(&at(&i, 0, 0), GridEvent::StartEdit, &i).state.editing.is_none());
        let s = run(&at(&i, 0, 1), GridEvent::StartEdit, &i);
        assert_eq!(s.state.editing, Some(Editing { row: 0, col: 1, draft: "1".into() }));
        assert_eq!(s.commands, [GridCommand::FocusEditor { row: 0, col: 1 }]);
    }

    #[test]
    fn enter_saves_escape_discards() {
        let i = input();
        let s = run(&at(&i, 3, 1), GridEvent::StartEdit, &i).state;
        let s = run(&s, GridEvent::EditChange("42".into()), &i).state;
        assert!(run(&s, GridEvent::Move(Nav::Down), &i).commands.is_empty(), "keys do not navigate while editing");
        let saved = run(&s, GridEvent::Commit, &i);
        assert_eq!(saved.state.rows[3].cells[1], "42");
        assert!(saved.state.editing.is_none());
        assert_eq!(
            saved.commands[0],
            GridCommand::Edit(GridEdit { row: "r3".into(), column: "qty".into(), value: "42".into(), previous: "1".into() })
        );
        assert_eq!(saved.commands[1], GridCommand::FocusCell { row: 3, col: 1 });
        let dropped = run(&s, GridEvent::Cancel, &i);
        assert_eq!(dropped.state.rows[3].cells[1], "1");
        assert_eq!(dropped.commands, [GridCommand::FocusCell { row: 3, col: 1 }]);
    }

    #[test]
    fn blur_saves_without_stealing_focus_and_unchanged_edits_are_silent() {
        let i = input();
        let s = run(&at(&i, 0, 1), GridEvent::StartEdit, &i).state;
        assert!(run(&s, GridEvent::Blur, &i).commands.is_empty());
        let s = run(&s, GridEvent::EditChange("9".into()), &i).state;
        let blurred = run(&s, GridEvent::Blur, &i);
        assert!(matches!(blurred.commands.as_slice(), [GridCommand::Edit(_)]));
    }

    #[test]
    fn controlled_rows_wait_for_the_owner() {
        let mut i = input();
        i.rows = Some(i.default_rows.clone());
        let s = GridMachine::initial(&i);
        let s = run(&run(&s, GridEvent::StartEditAt { row: 0, col: 1 }, &i).state, GridEvent::EditChange("7".into()), &i).state;
        let done = run(&s, GridEvent::Commit, &i);
        assert_eq!(done.state.rows[0].cells[1], "1");
        assert!(matches!(done.commands[0], GridCommand::Edit(_)));
    }

    #[test]
    fn sync_drops_focus_outside_new_rows() {
        let mut i = input();
        i.rows = Some(i.default_rows.clone());
        let s = at(&i, 20, 0);
        i.rows = Some(i.default_rows[..5].to_vec());
        assert_eq!(hydroxy_statechart::sync_or_keep::<GridMachine>(&GridState { controlled: true, ..s }, &i).focus, None);
    }
}
