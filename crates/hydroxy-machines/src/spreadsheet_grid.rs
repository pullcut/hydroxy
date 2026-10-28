//! Spreadsheet grid: an editable sheet with A1 column letters and row numbers.
//! Click, Shift+click or drag to select a range; arrows move, Shift+arrows
//! extend. Enter, F2 or typing edits the active cell; Enter and Tab save and
//! move, Escape discards. Cells starting with `=` are [`formula`]s and are
//! recalculated in dependency order after every change. Ranges copy and paste
//! as tab-separated text, columns resize, and only the rows in view are
//! rendered: rows have a fixed height, so the window is plain arithmetic on
//! the scroll offset.

pub mod formula;

use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::sync::Arc;

use hydroxy_statechart::{Machine, Transition};

pub use formula::{Cell, CellError, Value, cell_name, column_name};

/// Height of every row, and of the column header row, in pixels.
pub const ROW_HEIGHT: f64 = 28.0;
/// Width of the row number column, in pixels.
pub const ROW_HEADER_WIDTH: f64 = 52.0;
/// Narrowest a column can be resized to.
pub const MIN_COL_WIDTH: f64 = 40.0;
/// Widest a column can be resized to.
pub const MAX_COL_WIDTH: f64 = 640.0;
/// Rows rendered beyond each edge of the viewport.
pub const OVERSCAN: usize = 6;

#[allow(clippy::cast_precision_loss, reason = "row and column counts are far below 2^52")]
fn px(n: usize) -> f64 {
    n as f64
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "clamped to be non-negative; pixel offsets are far below usize::MAX")]
fn index(x: f64) -> usize {
    x.max(0.0) as usize
}

/// Props for [`SpreadsheetGridMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct SpreadsheetGridInput {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
    /// Initial cell text by (row, column). Text starting with `=` is a formula.
    pub default_cells: Arc<BTreeMap<Cell, String>>,
    /// Initial width of every column, in pixels.
    pub col_width: f64,
    /// Height of the scrolling viewport, in pixels, header included.
    pub height: f64,
}

impl Default for SpreadsheetGridInput {
    fn default() -> Self {
        Self { rows: 100, cols: 8, default_cells: Arc::default(), col_width: 112.0, height: 360.0 }
    }
}

/// A rectangle of cells, inclusive on every side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// First row.
    pub top: usize,
    /// First column.
    pub left: usize,
    /// Last row.
    pub bottom: usize,
    /// Last column.
    pub right: usize,
}

impl Rect {
    /// The rectangle spanned by two corners.
    #[must_use]
    pub fn spanning(a: Cell, b: Cell) -> Self {
        Self { top: a.0.min(b.0), left: a.1.min(b.1), bottom: a.0.max(b.0), right: a.1.max(b.1) }
    }

    /// Whether `cell` is inside.
    #[must_use]
    pub fn contains(&self, (row, col): Cell) -> bool {
        (self.top..=self.bottom).contains(&row) && (self.left..=self.right).contains(&col)
    }

    /// Number of cells.
    #[must_use]
    pub fn count(&self) -> usize {
        (self.bottom - self.top + 1) * (self.right - self.left + 1)
    }

    /// `B2` for one cell, `B2:D5` for more.
    #[must_use]
    pub fn name(&self) -> String {
        let (a, b) = ((self.top, self.left), (self.bottom, self.right));
        if a == b { cell_name(a) } else { format!("{}:{}", cell_name(a), cell_name(b)) }
    }
}

/// A column resize in progress.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resizing {
    /// The column.
    pub col: usize,
    /// Pointer x when the drag started.
    pub start_x: f64,
    /// The column's width when the drag started.
    pub start_width: f64,
}

/// One cell's text changing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellChange {
    /// Row, 0-based.
    pub row: usize,
    /// Column, 0-based.
    pub col: usize,
    /// The text before.
    pub previous: String,
    /// The new text; empty when cleared.
    pub value: String,
}

/// What [`SpreadsheetGridMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq)]
pub struct SpreadsheetGridState {
    /// Raw cell text, formulas included. Shared, so a transition that does not
    /// edit does not copy the sheet.
    pub cells: Arc<BTreeMap<Cell, String>>,
    /// The value of every formula cell.
    pub values: Arc<HashMap<Cell, Value>>,
    /// The active cell: where typing goes, and one corner of the selection.
    pub active: Cell,
    /// The selection's other corner. Equal to `active` for a single cell.
    pub extent: Cell,
    /// A pointer drag is selecting.
    pub selecting: bool,
    /// Editor text while the active cell is being edited.
    pub editing: Option<String>,
    /// Column widths in pixels.
    pub widths: Vec<f64>,
    /// A column resize drag.
    pub resizing: Option<Resizing>,
    /// Viewport scroll offsets and size, in pixels.
    pub scroll_top: f64,
    /// Horizontal scroll offset.
    pub scroll_left: f64,
    /// Viewport height, header included.
    pub viewport_height: f64,
    /// Viewport width, row numbers included. 0 until measured.
    pub viewport_width: f64,
    /// Text for the polite live region.
    pub status: String,
}

impl SpreadsheetGridState {
    /// The selected rectangle.
    #[must_use]
    pub fn selection(&self) -> Rect {
        Rect::spanning(self.active, self.extent)
    }

    /// A cell's raw text.
    #[must_use]
    pub fn raw(&self, cell: Cell) -> &str {
        self.cells.get(&cell).map_or("", String::as_str)
    }

    /// A cell's value: the formula result, or the text read as a number or text.
    #[must_use]
    pub fn value(&self, cell: Cell) -> Value {
        let raw = self.raw(cell);
        if raw.starts_with('=') { self.values.get(&cell).cloned().unwrap_or(Value::Empty) } else { formula::literal(raw) }
    }

    /// The text shown in a cell. Plain cells show what was typed.
    #[must_use]
    pub fn display(&self, cell: Cell) -> String {
        let raw = self.raw(cell);
        if raw.starts_with('=') { self.value(cell).display() } else { raw.to_string() }
    }

    /// Rows to render: those in view plus [`OVERSCAN`] each side.
    #[must_use]
    pub fn visible_rows(&self, input: &SpreadsheetGridInput) -> Range<usize> {
        let first = index(self.scroll_top / ROW_HEIGHT).saturating_sub(OVERSCAN);
        let last = index(((self.scroll_top + self.viewport_height) / ROW_HEIGHT).ceil()) + OVERSCAN;
        first.min(input.rows)..last.min(input.rows)
    }

    /// Rows that fit in the viewport below the header.
    #[must_use]
    pub fn page_rows(&self) -> usize {
        index((self.viewport_height - ROW_HEIGHT) / ROW_HEIGHT).max(1)
    }

    /// Left edge of a column in content pixels (row numbers come first).
    #[must_use]
    pub fn column_left(&self, col: usize) -> f64 {
        ROW_HEADER_WIDTH + self.widths.iter().take(col).sum::<f64>()
    }

    /// Width of all columns, row numbers included.
    #[must_use]
    pub fn total_width(&self) -> f64 {
        self.column_left(self.widths.len())
    }

    /// The selection as tab-separated raw text (formulas stay formulas).
    #[must_use]
    pub fn selection_tsv(&self) -> String {
        let r = self.selection();
        let rows: Vec<Vec<String>> =
            (r.top..=r.bottom).map(|row| (r.left..=r.right).map(|col| self.raw((row, col)).to_string()).collect()).collect();
        to_tsv(&rows)
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
    /// First row, same column.
    Top,
    /// Last row, same column.
    Bottom,
    /// First column of the row.
    RowStart,
    /// Last column of the row.
    RowEnd,
    /// A1.
    SheetStart,
    /// The last cell of the sheet.
    SheetEnd,
    /// Up one screen.
    PageUp,
    /// Down one screen.
    PageDown,
}

/// Where the active cell goes after an edit is saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum After {
    /// Enter: down.
    Down,
    /// Shift+Enter: up.
    Up,
    /// Tab: right.
    Right,
    /// Shift+Tab: left.
    Left,
    /// Stay put.
    Stay,
}

/// Inputs to [`SpreadsheetGridMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum SpreadsheetGridEvent {
    /// Primary button down on a cell. Shift extends the selection.
    PointerDown {
        /// The cell.
        cell: Cell,
        /// Shift was held.
        shift: bool,
    },
    /// The pointer moved onto a cell with the button held.
    PointerEnter(Cell),
    /// The button was released.
    PointerUp,
    /// Double click: edit this cell.
    EditAt(Cell),
    /// A navigation key. `extend` (Shift) moves the selection's far corner.
    Move {
        /// Where to go.
        nav: Nav,
        /// Extend the selection instead of moving.
        extend: bool,
    },
    /// Ctrl+A.
    SelectAll,
    /// Escape outside the editor: shrink the selection to the active cell.
    Collapse,
    /// Start editing the active cell. `Some(text)` replaces its content (typing);
    /// `None` keeps it (Enter, F2).
    StartEdit(Option<String>),
    /// The editor's text changed.
    EditChange(String),
    /// Save the edit, move, and focus the grid.
    Commit(After),
    /// The editor lost focus: save without moving focus.
    Blur,
    /// Escape in the editor: discard.
    Cancel,
    /// Delete or Backspace: empty every selected cell.
    Clear,
    /// Tab-separated text was pasted.
    Paste(String),
    /// The viewport scrolled or was measured.
    Scroll {
        /// `scrollTop`.
        top: f64,
        /// `scrollLeft`.
        left: f64,
        /// `clientWidth`.
        width: f64,
        /// `clientHeight`.
        height: f64,
    },
    /// Pointer down on a column's resize handle.
    ResizeStart {
        /// The column.
        col: usize,
        /// Pointer x.
        x: f64,
    },
    /// The resize pointer moved.
    ResizeMove(f64),
    /// The resize pointer was released.
    ResizeEnd,
    /// Resize by keyboard.
    ResizeBy {
        /// The column.
        col: usize,
        /// Pixels to add (negative to shrink).
        delta: f64,
    },
}

/// Side effects from [`SpreadsheetGridMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum SpreadsheetGridCommand {
    /// Move DOM focus to the grid (after the editor closes).
    FocusGrid,
    /// Focus the editor once it mounts, caret at the end.
    FocusEditor,
    /// Set the viewport's scroll offsets so the active cell is in view.
    ScrollTo {
        /// `scrollTop`.
        top: f64,
        /// `scrollLeft`.
        left: f64,
    },
    /// Cells changed, in order. One edit, a paste or a clear.
    Change(Vec<CellChange>),
}

/// Selection, editing, formulas, clipboard and windowing for a spreadsheet.
#[derive(Debug, Clone, Copy)]
pub struct SpreadsheetGridMachine;

type Step = Transition<SpreadsheetGridState, SpreadsheetGridCommand>;

fn clamp_cell((row, col): Cell, input: &SpreadsheetGridInput) -> Cell {
    (row.min(input.rows.saturating_sub(1)), col.min(input.cols.saturating_sub(1)))
}

fn target((r, c): Cell, nav: Nav, input: &SpreadsheetGridInput, page: usize) -> Cell {
    let (last_r, last_c) = (input.rows.saturating_sub(1), input.cols.saturating_sub(1));
    match nav {
        Nav::Up => (r.saturating_sub(1), c),
        Nav::Down => ((r + 1).min(last_r), c),
        Nav::Left => (r, c.saturating_sub(1)),
        Nav::Right => (r, (c + 1).min(last_c)),
        Nav::Top => (0, c),
        Nav::Bottom => (last_r, c),
        Nav::RowStart => (r, 0),
        Nav::RowEnd => (r, last_c),
        Nav::SheetStart => (0, 0),
        Nav::SheetEnd => (last_r, last_c),
        Nav::PageUp => (r.saturating_sub(page), c),
        Nav::PageDown => ((r + page).min(last_r), c),
    }
}

/// Scroll `next` so `cell` is fully visible below the sticky header and right
/// of the sticky row numbers. `Some` command when the offsets changed.
fn reveal(next: &mut SpreadsheetGridState, cell: Cell) -> Option<SpreadsheetGridCommand> {
    let body_h = (next.viewport_height - ROW_HEIGHT).max(ROW_HEIGHT);
    let y = px(cell.0) * ROW_HEIGHT;
    let mut top = next.scroll_top;
    if y < top {
        top = y;
    } else if y + ROW_HEIGHT > top + body_h {
        top = y + ROW_HEIGHT - body_h;
    }
    let mut left = next.scroll_left;
    if next.viewport_width > 0.0
        && let Some(&w) = next.widths.get(cell.1)
    {
        let x = next.column_left(cell.1) - ROW_HEADER_WIDTH;
        let body_w = (next.viewport_width - ROW_HEADER_WIDTH).max(w);
        if x < left {
            left = x;
        } else if x + w > left + body_w {
            left = x + w - body_w;
        }
    }
    if (top - next.scroll_top).abs() < 0.5 && (left - next.scroll_left).abs() < 0.5 {
        return None;
    }
    next.scroll_top = top;
    next.scroll_left = left;
    Some(SpreadsheetGridCommand::ScrollTo { top, left })
}

fn selection_status(state: &SpreadsheetGridState) -> String {
    let r = state.selection();
    if r.count() == 1 { String::new() } else { format!("{} selected, {} cells", r.name(), r.count()) }
}

/// Write cells into `next`, recalculating once. Returns what actually changed.
fn write(next: &mut SpreadsheetGridState, input: &SpreadsheetGridInput, edits: Vec<(Cell, String)>) -> Vec<CellChange> {
    let mut changes = Vec::new();
    let cells = Arc::make_mut(&mut next.cells);
    for ((row, col), value) in edits {
        if row >= input.rows || col >= input.cols {
            continue;
        }
        let previous = cells.get(&(row, col)).cloned().unwrap_or_default();
        let value = if value.trim().is_empty() { String::new() } else { value };
        if previous == value {
            continue;
        }
        if value.is_empty() {
            cells.remove(&(row, col));
        } else {
            cells.insert((row, col), value.clone());
        }
        changes.push(CellChange { row, col, previous, value });
    }
    if !changes.is_empty() {
        // full recalculation per write (a few ms for ~10k formulas); track
        // dependents and recalc only the changed cone if sheets grow far larger.
        next.values = Arc::new(formula::recalc(&next.cells, (input.rows, input.cols)));
    }
    changes
}

fn commit(state: &SpreadsheetGridState, input: &SpreadsheetGridInput, after: After, refocus: bool) -> Step {
    let Some(draft) = &state.editing else { return Transition::stay() };
    let mut next = state.clone();
    next.editing = None;
    let mut commands = Vec::new();
    let cell = state.active;
    let changes = write(&mut next, input, vec![(cell, draft.clone())]);
    next.status = if draft.starts_with('=') { format!("{} = {}", cell_name(cell), next.display(cell)) } else { String::new() };
    if !changes.is_empty() {
        commands.push(SpreadsheetGridCommand::Change(changes));
    }
    let nav = match after {
        After::Down => Some(Nav::Down),
        After::Up => Some(Nav::Up),
        After::Right => Some(Nav::Right),
        After::Left => Some(Nav::Left),
        After::Stay => None,
    };
    if let Some(nav) = nav {
        let to = target(cell, nav, input, 1);
        next.active = to;
        next.extent = to;
        commands.extend(reveal(&mut next, to));
    }
    if refocus {
        commands.push(SpreadsheetGridCommand::FocusGrid);
    }
    Transition::with(next, commands)
}

fn start_edit(state: &SpreadsheetGridState, text: Option<String>) -> Step {
    if state.editing.is_some() {
        return Transition::stay();
    }
    let mut next = state.clone();
    next.editing = Some(text.unwrap_or_else(|| state.raw(state.active).to_string()));
    let mut commands: Vec<_> = reveal(&mut next, state.active).into_iter().collect();
    commands.push(SpreadsheetGridCommand::FocusEditor);
    Transition::with(next, commands)
}

fn paste(state: &SpreadsheetGridState, input: &SpreadsheetGridInput, text: &str) -> Step {
    let grid = parse_tsv(text);
    if grid.is_empty() || state.editing.is_some() {
        return Transition::stay();
    }
    let sel = state.selection();
    let mut next = state.clone();
    let mut edits = Vec::new();
    let single = grid.len() == 1 && grid[0].len() == 1;
    let area = if single && sel.count() > 1 {
        // One value into a range fills the range.
        for row in sel.top..=sel.bottom {
            edits.extend((sel.left..=sel.right).map(|col| ((row, col), grid[0][0].clone())));
        }
        sel
    } else {
        let width = grid.iter().map(Vec::len).max().unwrap_or(1);
        for (dr, line) in grid.iter().enumerate() {
            edits.extend(line.iter().enumerate().map(|(dc, v)| ((sel.top + dr, sel.left + dc), v.clone())));
        }
        let far = clamp_cell((sel.top + grid.len() - 1, sel.left + width - 1), input);
        Rect::spanning((sel.top, sel.left), far)
    };
    let changes = write(&mut next, input, edits);
    next.active = (area.top, area.left);
    next.extent = (area.bottom, area.right);
    next.status = format!("Pasted into {}", area.name());
    let commands = if changes.is_empty() { Vec::new() } else { vec![SpreadsheetGridCommand::Change(changes)] };
    Transition::with(next, commands)
}

fn resize(state: &SpreadsheetGridState, col: usize, width: f64) -> Step {
    let width = width.clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
    match state.widths.get(col) {
        Some(w) if (w - width).abs() >= 0.5 => {
            let mut next = state.clone();
            next.widths[col] = width;
            Transition::to(next)
        }
        _ => Transition::stay(),
    }
}

impl Machine for SpreadsheetGridMachine {
    type Input = SpreadsheetGridInput;
    type State = SpreadsheetGridState;
    type Event = SpreadsheetGridEvent;
    type Command = SpreadsheetGridCommand;

    fn initial(input: &SpreadsheetGridInput) -> SpreadsheetGridState {
        let cells = Arc::clone(&input.default_cells);
        SpreadsheetGridState {
            values: Arc::new(formula::recalc(&cells, (input.rows, input.cols))),
            cells,
            active: (0, 0),
            extent: (0, 0),
            selecting: false,
            editing: None,
            widths: vec![input.col_width; input.cols],
            resizing: None,
            scroll_top: 0.0,
            scroll_left: 0.0,
            viewport_height: input.height,
            viewport_width: 0.0,
            status: String::new(),
        }
    }

    #[allow(clippy::too_many_lines, reason = "one match over every event")]
    fn transition(state: &SpreadsheetGridState, event: SpreadsheetGridEvent, input: &SpreadsheetGridInput) -> Step {
        if input.rows == 0 || input.cols == 0 {
            return Transition::stay();
        }
        let editing = state.editing.is_some();
        match event {
            SpreadsheetGridEvent::PointerDown { cell, shift } => {
                let cell = clamp_cell(cell, input);
                // A click elsewhere saves the edit first.
                let step = commit(state, input, After::Stay, false).resolve(state);
                let (mut next, commands) = (step.state, step.commands);
                if shift {
                    next.extent = cell;
                } else {
                    next.active = cell;
                    next.extent = cell;
                }
                next.selecting = true;
                // Keep the saved formula's result as the status unless a range is selected.
                let status = selection_status(&next);
                if !status.is_empty() || !editing {
                    next.status = status;
                }
                Transition::with(next, commands)
            }
            SpreadsheetGridEvent::PointerEnter(cell) => {
                let cell = clamp_cell(cell, input);
                if !state.selecting || editing || state.extent == cell {
                    return Transition::stay();
                }
                let mut next = state.clone();
                next.extent = cell;
                next.status = selection_status(&next);
                Transition::to(next)
            }
            SpreadsheetGridEvent::PointerUp => {
                if !state.selecting {
                    return Transition::stay();
                }
                let mut next = state.clone();
                next.selecting = false;
                Transition::to(next)
            }
            SpreadsheetGridEvent::EditAt(cell) => {
                if editing {
                    return Transition::stay();
                }
                let mut moved = state.clone();
                moved.active = clamp_cell(cell, input);
                moved.extent = moved.active;
                moved.selecting = false;
                let step = start_edit(&moved, None).resolve(&moved);
                Transition::with(step.state, step.commands)
            }
            SpreadsheetGridEvent::Move { nav, extend } => {
                if editing {
                    return Transition::stay();
                }
                let from = if extend { state.extent } else { state.active };
                let to = target(from, nav, input, state.page_rows());
                let mut next = state.clone();
                if extend {
                    next.extent = to;
                    next.status = selection_status(&next);
                } else {
                    next.active = to;
                    next.extent = to;
                    next.status.clear();
                }
                if next.active == state.active && next.extent == state.extent {
                    return Transition::stay();
                }
                let commands = reveal(&mut next, to).into_iter().collect();
                Transition::with(next, commands)
            }
            SpreadsheetGridEvent::SelectAll => {
                let all = ((0, 0), (input.rows - 1, input.cols - 1));
                if editing || (state.active, state.extent) == all {
                    return Transition::stay();
                }
                let mut next = state.clone();
                (next.active, next.extent) = all;
                next.status = format!("All {} cells selected", next.selection().count());
                Transition::to(next)
            }
            SpreadsheetGridEvent::Collapse => {
                if editing || state.extent == state.active {
                    return Transition::stay();
                }
                let mut next = state.clone();
                next.extent = next.active;
                next.status.clear();
                Transition::to(next)
            }
            SpreadsheetGridEvent::StartEdit(text) => start_edit(state, text),
            SpreadsheetGridEvent::EditChange(text) => {
                if !editing {
                    return Transition::stay();
                }
                let mut next = state.clone();
                next.editing = Some(text);
                Transition::to(next)
            }
            SpreadsheetGridEvent::Commit(after) => commit(state, input, after, true),
            SpreadsheetGridEvent::Blur => commit(state, input, After::Stay, false),
            SpreadsheetGridEvent::Cancel => {
                if !editing {
                    return Transition::stay();
                }
                let mut next = state.clone();
                next.editing = None;
                Transition::with(next, vec![SpreadsheetGridCommand::FocusGrid])
            }
            SpreadsheetGridEvent::Clear => {
                if editing {
                    return Transition::stay();
                }
                let sel = state.selection();
                let edits: Vec<(Cell, String)> =
                    state.cells.keys().filter(|c| sel.contains(**c)).map(|c| (*c, String::new())).collect();
                let mut next = state.clone();
                let changes = write(&mut next, input, edits);
                if changes.is_empty() {
                    return Transition::stay();
                }
                next.status = format!("Cleared {}", sel.name());
                Transition::with(next, vec![SpreadsheetGridCommand::Change(changes)])
            }
            SpreadsheetGridEvent::Paste(text) => paste(state, input, &text),
            SpreadsheetGridEvent::Scroll { top, left, width, height } => {
                let near = |a: f64, b: f64| (a - b).abs() < 0.5;
                if near(top, state.scroll_top)
                    && near(left, state.scroll_left)
                    && near(width, state.viewport_width)
                    && near(height, state.viewport_height)
                {
                    return Transition::stay();
                }
                let mut next = state.clone();
                (next.scroll_top, next.scroll_left, next.viewport_width, next.viewport_height) = (top, left, width, height);
                Transition::to(next)
            }
            SpreadsheetGridEvent::ResizeStart { col, x } => {
                let Some(&start_width) = state.widths.get(col) else { return Transition::stay() };
                let mut next = state.clone();
                next.resizing = Some(Resizing { col, start_x: x, start_width });
                Transition::to(next)
            }
            SpreadsheetGridEvent::ResizeMove(x) => {
                let Some(r) = state.resizing else { return Transition::stay() };
                resize(state, r.col, r.start_width + x - r.start_x)
            }
            SpreadsheetGridEvent::ResizeEnd => {
                if state.resizing.is_none() {
                    return Transition::stay();
                }
                let mut next = state.clone();
                next.resizing = None;
                Transition::to(next)
            }
            SpreadsheetGridEvent::ResizeBy { col, delta } => match state.widths.get(col) {
                Some(w) => resize(state, col, w + delta),
                None => Transition::stay(),
            },
        }
    }

    fn sync(state: &SpreadsheetGridState, input: &SpreadsheetGridInput) -> Option<SpreadsheetGridState> {
        let (active, extent) = (clamp_cell(state.active, input), clamp_cell(state.extent, input));
        if active == state.active && extent == state.extent && state.widths.len() == input.cols {
            return None;
        }
        let mut next = state.clone();
        (next.active, next.extent) = (active, extent);
        next.widths.resize(input.cols, input.col_width);
        Arc::make_mut(&mut next.cells).retain(|&(r, c), _| r < input.rows && c < input.cols);
        next.values = Arc::new(formula::recalc(&next.cells, (input.rows, input.cols)));
        Some(next)
    }
}

/// Cells keyed by A1 name, for initial content: `[("A1", "Rent"), ("B1", "1200")]`.
/// Names that do not parse are skipped.
#[must_use]
pub fn cells_from_a1(pairs: &[(&str, &str)]) -> BTreeMap<Cell, String> {
    pairs.iter().filter_map(|(name, text)| formula::parse_cell(name).map(|c| (c, (*text).to_string()))).collect()
}

/// Rows of fields from tab-separated text. Fields in double quotes may hold
/// tabs, newlines and doubled quotes, as spreadsheets write them. A final
/// newline does not add an empty row.
#[must_use]
pub fn parse_tsv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c != '"' {
                field.push(c);
            } else if chars.peek() == Some(&'"') {
                chars.next();
                field.push('"');
            } else {
                quoted = false;
            }
            continue;
        }
        match c {
            '"' if field.is_empty() => quoted = true,
            '\t' => row.push(std::mem::take(&mut field)),
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            _ => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// Tab-separated text from rows of fields, quoting fields that need it.
#[must_use]
pub fn to_tsv(rows: &[Vec<String>]) -> String {
    let quote = |f: &String| {
        if f.contains(['\t', '\n', '"']) { format!("\"{}\"", f.replace('"', "\"\"")) } else { f.clone() }
    };
    rows.iter().map(|r| r.iter().map(quote).collect::<Vec<_>>().join("\t")).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{SpreadsheetGridCommand as C, SpreadsheetGridEvent as E};

    fn input() -> SpreadsheetGridInput {
        let cells: BTreeMap<Cell, String> = [((0, 0), "10"), ((1, 0), "20"), ((2, 0), "=SUM(A1:A2)"), ((0, 1), "note")]
            .into_iter()
            .map(|(c, v)| (c, v.to_string()))
            .collect();
        SpreadsheetGridInput { rows: 10_000, cols: 6, default_cells: Arc::new(cells), col_width: 100.0, height: 28.0 * 11.0 }
    }

    fn run(state: &SpreadsheetGridState, event: E, input: &SpreadsheetGridInput) -> hydroxy_statechart::Step<SpreadsheetGridState, C> {
        SpreadsheetGridMachine::transition(state, event, input).resolve(state)
    }

    fn start() -> (SpreadsheetGridState, SpreadsheetGridInput) {
        let i = input();
        (SpreadsheetGridMachine::initial(&i), i)
    }

    #[test]
    fn formulas_are_calculated_from_the_start() {
        let (s, _) = start();
        assert_eq!(s.display((2, 0)), "30");
        assert_eq!(s.raw((2, 0)), "=SUM(A1:A2)");
        assert_eq!(s.value((0, 1)), Value::Text("note".into()));
    }

    #[test]
    fn arrows_move_and_shift_arrows_extend() {
        let (s, i) = start();
        let s = run(&s, E::Move { nav: Nav::Right, extend: false }, &i).state;
        assert_eq!((s.active, s.extent), ((0, 1), (0, 1)));
        let s = run(&s, E::Move { nav: Nav::Down, extend: true }, &i).state;
        let s = run(&s, E::Move { nav: Nav::Right, extend: true }, &i).state;
        assert_eq!(s.selection(), Rect { top: 0, left: 1, bottom: 1, right: 2 });
        assert_eq!(s.active, (0, 1), "the active cell stays put while extending");
        assert_eq!(s.status, "B1:C2 selected, 4 cells");
        let s = run(&s, E::Move { nav: Nav::Left, extend: false }, &i).state;
        assert_eq!((s.active, s.extent), ((0, 0), (0, 0)), "a plain arrow collapses");
        assert!(run(&s, E::Move { nav: Nav::Up, extend: false }, &i).commands.is_empty());
        assert_eq!(run(&s, E::Move { nav: Nav::SheetEnd, extend: false }, &i).state.active, (9_999, 5));
        assert_eq!(run(&s, E::Move { nav: Nav::PageDown, extend: false }, &i).state.active, (10, 0));
    }

    #[test]
    fn moving_off_screen_scrolls_and_windows_the_rows() {
        let (s, i) = start();
        assert_eq!(s.visible_rows(&i), 0..17);
        let step = run(&s, E::Move { nav: Nav::Bottom, extend: false }, &i);
        assert_eq!(step.commands, [C::ScrollTo { top: 28.0 * 9_990.0, left: 0.0 }]);
        assert_eq!(step.state.visible_rows(&i), 9_984..10_000);
        let back = run(&step.state, E::Move { nav: Nav::Top, extend: false }, &i);
        assert_eq!(back.commands, [C::ScrollTo { top: 0.0, left: 0.0 }]);
        let scrolled = run(&s, E::Scroll { top: 2800.0, left: 0.0, width: 400.0, height: 308.0 }, &i).state;
        assert_eq!(scrolled.visible_rows(&i), 94..117);
        // Column 5 sits past a 400px viewport: scroll right just enough.
        let right = run(&scrolled, E::Move { nav: Nav::RowEnd, extend: false }, &i);
        assert_eq!(right.commands, [C::ScrollTo { top: 0.0, left: 252.0 }]);
    }

    #[test]
    fn pointer_click_shift_click_and_drag_select() {
        let (s, i) = start();
        let s = run(&s, E::PointerDown { cell: (3, 1), shift: false }, &i).state;
        let s = run(&s, E::PointerEnter((5, 2)), &i).state;
        assert_eq!(s.selection(), Rect { top: 3, left: 1, bottom: 5, right: 2 });
        let s = run(&s, E::PointerUp, &i).state;
        assert_eq!(run(&s, E::PointerEnter((9, 4)), &i).state.extent, (5, 2), "no drag after release");
        let s = run(&s, E::PointerDown { cell: (0, 0), shift: true }, &i).state;
        assert_eq!(s.selection(), Rect { top: 0, left: 0, bottom: 3, right: 1 });
        assert_eq!(s.active, (3, 1));
    }

    #[test]
    fn typing_replaces_and_enter_saves_then_moves_down() {
        let (s, i) = start();
        let s = run(&s, E::Move { nav: Nav::Right, extend: false }, &i).state;
        let step = run(&s, E::StartEdit(Some("=".into())), &i);
        assert_eq!(step.commands, [C::FocusEditor]);
        let s = run(&step.state, E::EditChange("=A3*2".into()), &i).state;
        assert!(run(&s, E::Move { nav: Nav::Down, extend: false }, &i).commands.is_empty(), "keys belong to the editor");
        let done = run(&s, E::Commit(After::Down), &i);
        assert_eq!(done.state.display((0, 1)), "60");
        assert_eq!(done.state.active, (1, 1));
        assert_eq!(done.state.status, "B1 = 60");
        assert_eq!(
            done.commands,
            [
                C::Change(vec![CellChange { row: 0, col: 1, previous: "note".into(), value: "=A3*2".into() }]),
                C::FocusGrid
            ]
        );
    }

    #[test]
    fn f2_keeps_content_tab_moves_right_escape_discards() {
        let (s, i) = start();
        let s = run(&s, E::StartEdit(None), &i).state;
        assert_eq!(s.editing.as_deref(), Some("10"));
        let s = run(&s, E::EditChange("15".into()), &i).state;
        let tabbed = run(&s, E::Commit(After::Right), &i).state;
        assert_eq!((tabbed.active, tabbed.display((2, 0))), ((0, 1), "35".to_string()), "dependents recalculate");
        let dropped = run(&s, E::Cancel, &i);
        assert_eq!(dropped.state.raw((0, 0)), "10");
        assert_eq!(dropped.commands, [C::FocusGrid]);
    }

    #[test]
    fn clicking_away_saves_and_blur_does_not_steal_focus() {
        let (s, i) = start();
        let s = run(&run(&s, E::StartEdit(Some("7".into())), &i).state, E::EditChange("7".into()), &i).state;
        let clicked = run(&s, E::PointerDown { cell: (4, 4), shift: false }, &i);
        assert_eq!(clicked.state.raw((0, 0)), "7");
        assert_eq!(clicked.state.active, (4, 4));
        assert!(matches!(clicked.commands.as_slice(), [C::Change(_)]));
        let blurred = run(&s, E::Blur, &i);
        assert!(matches!(blurred.commands.as_slice(), [C::Change(_)]));
        assert!(run(&blurred.state, E::Blur, &i).commands.is_empty());
    }

    #[test]
    fn double_click_edits_that_cell() {
        let (s, i) = start();
        let step = run(&s, E::EditAt((2, 0)), &i);
        assert_eq!(step.state.active, (2, 0));
        assert_eq!(step.state.editing.as_deref(), Some("=SUM(A1:A2)"));
    }

    #[test]
    fn clear_empties_the_selection_and_reports_it() {
        let (s, i) = start();
        let s = run(&s, E::Move { nav: Nav::Down, extend: true }, &i).state;
        let step = run(&s, E::Clear, &i);
        assert_eq!(step.state.display((2, 0)), "0");
        assert_eq!(step.state.status, "Cleared A1:A2");
        let C::Change(changes) = &step.commands[0] else { panic!("expected a change") };
        assert_eq!(changes.len(), 2);
        assert!(run(&step.state, E::Clear, &i).commands.is_empty(), "nothing left to clear");
    }

    #[test]
    fn copy_and_paste_round_trip_as_tsv() {
        let (s, i) = start();
        let s = run(&run(&s, E::Move { nav: Nav::Down, extend: true }, &i).state, E::Move { nav: Nav::Right, extend: true }, &i).state;
        assert_eq!(s.selection_tsv(), "10\tnote\n20\t");
        let s = run(&s, E::PointerDown { cell: (5, 2), shift: false }, &i).state;
        let pasted = run(&s, E::Paste("1\t2\n3\t=C6+D6+C7\n".into()), &i);
        assert_eq!(pasted.state.display((6, 3)), "6");
        assert_eq!(pasted.state.selection(), Rect { top: 5, left: 2, bottom: 6, right: 3 });
        assert_eq!(pasted.state.status, "Pasted into C6:D7");
        let C::Change(changes) = &pasted.commands[0] else { panic!("expected a change") };
        assert_eq!(changes.len(), 4);
    }

    #[test]
    fn one_value_fills_a_range_and_pastes_clip_at_the_edge() {
        let (s, i) = start();
        let s = run(&s, E::PointerDown { cell: (0, 3), shift: false }, &i).state;
        let s = run(&s, E::PointerDown { cell: (2, 4), shift: true }, &i).state;
        let filled = run(&s, E::Paste("x".into()), &i).state;
        assert!((0..3).all(|r| (3..5).all(|c| filled.raw((r, c)) == "x")));
        let edge = run(&s, E::PointerDown { cell: (0, 5), shift: false }, &i).state;
        let clipped = run(&edge, E::Paste("a\tb\tc".into()), &i).state;
        assert_eq!(clipped.raw((0, 5)), "a");
        assert_eq!(clipped.selection(), Rect { top: 0, left: 5, bottom: 0, right: 5 });
    }

    #[test]
    fn a1_names_seed_cells() {
        let cells = cells_from_a1(&[("A1", "Rent"), ("c2", "=B2*2"), ("nope", "x")]);
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[&(1, 2)], "=B2*2");
    }

    #[test]
    fn tsv_handles_quotes_tabs_and_newlines() {
        let rows = vec![vec!["a\tb".to_string(), "say \"hi\"".to_string()], vec!["line\nbreak".to_string(), String::new()]];
        let text = to_tsv(&rows);
        assert_eq!(parse_tsv(&text), rows);
        assert_eq!(parse_tsv("1\t2\r\n3\t4\r\n"), [["1", "2"], ["3", "4"]]);
        assert!(parse_tsv("").is_empty());
    }

    #[test]
    fn select_all_and_escape() {
        let (s, i) = start();
        let all = run(&s, E::SelectAll, &i).state;
        assert_eq!(all.selection().count(), 60_000);
        assert_eq!(all.status, "All 60000 cells selected");
        assert_eq!(run(&all, E::Collapse, &i).state.selection().count(), 1);
    }

    #[test]
    fn columns_resize_by_drag_and_keyboard_within_limits() {
        let (s, i) = start();
        let s = run(&s, E::ResizeStart { col: 1, x: 300.0 }, &i).state;
        let s = run(&s, E::ResizeMove(350.0), &i).state;
        assert!((s.widths[1] - 150.0).abs() < f64::EPSILON);
        let s = run(&s, E::ResizeMove(0.0), &i).state;
        assert!((s.widths[1] - MIN_COL_WIDTH).abs() < f64::EPSILON);
        let s = run(&s, E::ResizeEnd, &i).state;
        assert!(run(&s, E::ResizeMove(500.0), &i).state.resizing.is_none());
        let s = run(&s, E::ResizeBy { col: 0, delta: 16.0 }, &i).state;
        assert!((s.widths[0] - 116.0).abs() < f64::EPSILON);
        assert!((s.column_left(1) - (ROW_HEADER_WIDTH + 116.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn sync_clamps_to_a_smaller_sheet() {
        let (s, mut i) = start();
        let s = run(&s, E::Move { nav: Nav::SheetEnd, extend: false }, &i).state;
        i.rows = 2;
        i.cols = 2;
        let synced = hydroxy_statechart::sync_or_keep::<SpreadsheetGridMachine>(&s, &i);
        assert_eq!(synced.active, (1, 1));
        assert_eq!(synced.widths.len(), 2);
        assert_eq!(synced.display((2, 0)), "", "rows past the end are dropped");
        assert_eq!(SpreadsheetGridMachine::sync(&synced, &i), None);
    }
}
