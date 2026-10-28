//! Dashboard layout: widget tiles on a column grid that move and resize in
//! grid units. A gesture (pointer drag, or keyboard pick-up with Enter or
//! Space) shows a live preview; a drop commits it and a cancel discards it.
//!
//! The layout maths is [`place`]: pure, with no DOM. A tile that lands on
//! others pushes them down, then everything floats back up to close gaps.

use hydroxy_statechart::{Machine, Transition};

/// One tile's cell rectangle. `x` and `y` are zero-based; `w` and `h` are at least 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardItem {
    /// Unique key.
    pub id: String,
    /// Column of the left edge.
    pub x: u32,
    /// Row of the top edge.
    pub y: u32,
    /// Width in columns.
    pub w: u32,
    /// Height in rows.
    pub h: u32,
}

impl DashboardItem {
    /// A tile at `(x, y)` that is `w` columns by `h` rows.
    #[must_use]
    pub fn new(id: impl Into<String>, x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { id: id.into(), x, y, w, h }
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.x < other.x + other.w && other.x < self.x + self.w && self.y < other.y + other.h && other.y < self.y + self.h
    }
}

fn clamp_cell(v: i32, min: i32, max: i32) -> u32 {
    u32::try_from(v.clamp(min, max.max(min))).unwrap_or(0)
}

/// Put tile `id` at the requested cell rectangle (clamped into `columns`) and
/// settle the others around it. Unknown ids leave the layout alone.
/// Output order matches the input.
#[must_use]
pub fn place(items: &[DashboardItem], id: &str, x: i32, y: i32, w: i32, h: i32, columns: u32) -> Vec<DashboardItem> {
    let Some(at) = items.iter().position(|i| i.id == id) else { return items.to_vec() };
    let columns = columns.max(1);
    let cols = i32::try_from(columns).unwrap_or(i32::MAX);
    let w = clamp_cell(w, 1, cols);
    let moved = DashboardItem {
        id: id.to_string(),
        w,
        h: clamp_cell(h, 1, i32::MAX),
        x: clamp_cell(x, 0, cols - i32::try_from(w).unwrap_or(0)),
        y: clamp_cell(y, 0, i32::MAX),
    };
    let mut out = items.to_vec();
    out[at] = moved.clone();
    let mut placed = vec![moved];
    let mut order: Vec<usize> = (0..items.len()).filter(|&i| i != at).collect();
    order.sort_by_key(|&i| (items[i].y, items[i].x));
    for i in order {
        let mut it = items[i].clone();
        it.w = it.w.clamp(1, columns);
        it.x = it.x.min(columns - it.w);
        let hits = |it: &DashboardItem| placed.iter().any(|p| p.overlaps(it));
        if hits(&it) {
            while hits(&it) {
                it.y += 1;
            }
        } else {
            while it.y > 0 {
                let up = DashboardItem { y: it.y - 1, ..it.clone() };
                if hits(&up) {
                    break;
                }
                it = up;
            }
        }
        placed.push(it.clone());
        out[i] = it;
    }
    out
}

/// Props for [`DashboardLayoutMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardInput {
    /// Controlled layout. `Some` at start fixes controlled mode.
    pub items: Option<Vec<DashboardItem>>,
    /// Uncontrolled initial layout.
    pub default_items: Vec<DashboardItem>,
    /// Columns in the grid.
    pub columns: u32,
    /// Block every gesture.
    pub disabled: bool,
}

impl Default for DashboardInput {
    fn default() -> Self {
        Self { items: None, default_items: Vec::new(), columns: 12, disabled: false }
    }
}

/// A tile being moved or resized, as offsets from where it started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gesture {
    /// The tile as it was when the gesture began.
    pub origin: DashboardItem,
    /// Cells moved right and down.
    pub shift: (i32, i32),
    /// Columns and rows added to the size.
    pub grow: (i32, i32),
    /// A pointer drag on the resize grip: drag deltas change the size, not the position.
    pub resizing: bool,
    /// Picked up with the keyboard rather than a pointer.
    pub keyboard: bool,
}

impl Gesture {
    /// The layout if the tile landed where the gesture asks.
    fn landed(&self, items: &[DashboardItem], columns: u32) -> Vec<DashboardItem> {
        let cell = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
        let o = &self.origin;
        place(
            items,
            &o.id,
            cell(o.x).saturating_add(self.shift.0),
            cell(o.y).saturating_add(self.shift.1),
            cell(o.w).saturating_add(self.grow.0),
            cell(o.h).saturating_add(self.grow.1),
            columns,
        )
    }
}

/// What [`DashboardLayoutMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DashboardState {
    /// The committed layout.
    pub items: Vec<DashboardItem>,
    /// The tile in flight.
    pub gesture: Option<Gesture>,
    /// Fixed at start: the owner holds the layout.
    pub controlled: bool,
}

impl DashboardState {
    /// The layout to render: the committed one, or the gesture's live preview.
    #[must_use]
    pub fn preview(&self, columns: u32) -> Vec<DashboardItem> {
        match &self.gesture {
            Some(g) => g.landed(&self.items, columns),
            None => self.items.clone(),
        }
    }

    /// The id of the tile in flight.
    #[must_use]
    pub fn active(&self) -> Option<&str> {
        self.gesture.as_ref().map(|g| g.origin.id.as_str())
    }
}

/// Inputs to [`DashboardLayoutMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardEvent {
    /// Enter or Space on a tile's move handle: pick it up.
    Grab(String),
    /// A pointer drag started on a tile's handle (`resize` false) or resize grip.
    DragStart {
        /// The tile.
        id: String,
        /// Started on the resize grip.
        resize: bool,
    },
    /// The pointer is this many cells from where the drag began.
    DragTo {
        /// Columns right.
        dx: i32,
        /// Rows down.
        dy: i32,
    },
    /// Arrow key while carrying: one cell. `resize` (Shift) changes the size.
    Nudge {
        /// Columns right.
        dx: i32,
        /// Rows down.
        dy: i32,
        /// Change the size instead of the position.
        resize: bool,
    },
    /// Enter or Space while carrying, or a pointer drop.
    Drop,
    /// Esc, or a pointer gesture that was interrupted.
    Cancel,
}

/// Where a tile is, in the 1-based numbers a screen reader should hear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spot {
    /// The tile's id.
    pub id: String,
    /// Left column, 1-based.
    pub column: u32,
    /// Top row, 1-based.
    pub row: u32,
    /// Width in columns.
    pub w: u32,
    /// Height in rows.
    pub h: u32,
}

/// What a screen reader should hear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Announcement {
    /// Picked up a tile.
    Grabbed(Spot),
    /// Carried to a new position or size.
    Moved(Spot),
    /// Dropped here.
    Dropped(Spot),
    /// Put back where it started.
    Cancelled(String),
}

/// Side effects from [`DashboardLayoutMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardCommand {
    /// The layout changed (or should, when controlled).
    LayoutChange(Vec<DashboardItem>),
    /// Say this in the live region.
    Announce(Announcement),
}

/// Layout and gesture state for a dashboard.
#[derive(Debug, Clone, Copy)]
pub struct DashboardLayoutMachine;

type Step = Transition<DashboardState, DashboardCommand>;

fn spot(items: &[DashboardItem], id: &str) -> Option<Spot> {
    items.iter().find(|i| i.id == id).map(|i| Spot { id: i.id.clone(), column: i.x + 1, row: i.y + 1, w: i.w, h: i.h })
}

fn begin(state: &DashboardState, id: &str, resizing: bool, keyboard: bool, columns: u32) -> Step {
    let Some(item) = state.items.iter().find(|i| i.id == id) else { return Transition::stay() };
    let mut next = state.clone();
    next.gesture = Some(Gesture { origin: item.clone(), shift: (0, 0), grow: (0, 0), resizing, keyboard });
    let commands = match spot(&next.preview(columns), id) {
        Some(s) if keyboard => vec![DashboardCommand::Announce(Announcement::Grabbed(s))],
        _ => Vec::new(),
    };
    Transition::with(next, commands)
}

impl Machine for DashboardLayoutMachine {
    type Input = DashboardInput;
    type State = DashboardState;
    type Event = DashboardEvent;
    type Command = DashboardCommand;

    fn initial(input: &DashboardInput) -> DashboardState {
        DashboardState {
            items: input.items.clone().unwrap_or_else(|| input.default_items.clone()),
            gesture: None,
            controlled: input.items.is_some(),
        }
    }

    fn transition(state: &DashboardState, event: DashboardEvent, input: &DashboardInput) -> Step {
        let stay = || Transition::stay();
        if input.disabled {
            return stay();
        }
        let columns = input.columns;
        match (event, &state.gesture) {
            (DashboardEvent::Grab(id), None) => begin(state, &id, false, true, columns),
            (DashboardEvent::DragStart { id, resize }, _) => begin(state, &id, resize, false, columns),
            (DashboardEvent::DragTo { dx, dy }, Some(g)) if !g.keyboard => {
                let mut next_g = g.clone();
                if g.resizing {
                    next_g.grow = (dx, dy);
                } else {
                    next_g.shift = (dx, dy);
                }
                if next_g == *g {
                    return stay();
                }
                let mut next = state.clone();
                next.gesture = Some(next_g);
                Transition::to(next)
            }
            (DashboardEvent::Nudge { dx, dy, resize }, Some(g)) if g.keyboard => {
                let mut next = state.clone();
                let mut next_g = g.clone();
                if resize {
                    next_g.grow = (next_g.grow.0 + dx, next_g.grow.1 + dy);
                } else {
                    next_g.shift = (next_g.shift.0 + dx, next_g.shift.1 + dy);
                }
                // Fold what clamping removed back in, so pressing against an edge does not store a debt.
                let landed = next_g.landed(&state.items, columns);
                let Some(at) = landed.iter().find(|i| i.id == g.origin.id) else { return stay() };
                let cell = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
                next_g.shift = (cell(at.x) - cell(g.origin.x), cell(at.y) - cell(g.origin.y));
                next_g.grow = (cell(at.w) - cell(g.origin.w), cell(at.h) - cell(g.origin.h));
                if next_g == *g {
                    return stay();
                }
                next.gesture = Some(next_g);
                let commands = spot(&landed, &g.origin.id).map(|s| DashboardCommand::Announce(Announcement::Moved(s)));
                Transition::with(next, commands.into_iter().collect())
            }
            (DashboardEvent::Drop, Some(g)) => {
                let landed = g.landed(&state.items, columns);
                let mut next = state.clone();
                next.gesture = None;
                let mut commands = Vec::new();
                if landed != state.items {
                    if !state.controlled {
                        next.items.clone_from(&landed);
                    }
                    commands.push(DashboardCommand::LayoutChange(landed.clone()));
                }
                if g.keyboard
                    && let Some(s) = spot(&landed, &g.origin.id)
                {
                    commands.push(DashboardCommand::Announce(Announcement::Dropped(s)));
                }
                Transition::with(next, commands)
            }
            (DashboardEvent::Cancel, Some(g)) => {
                let mut next = state.clone();
                next.gesture = None;
                let commands = if g.keyboard {
                    vec![DashboardCommand::Announce(Announcement::Cancelled(g.origin.id.clone()))]
                } else {
                    Vec::new()
                };
                Transition::with(next, commands)
            }
            _ => stay(),
        }
    }

    fn sync(state: &DashboardState, input: &DashboardInput) -> Option<DashboardState> {
        let mut next = state.clone();
        if state.controlled {
            next.items = input.items.clone().unwrap_or_default();
        }
        if input.disabled || next.gesture.as_ref().is_some_and(|g| !next.items.iter().any(|i| i.id == g.origin.id)) {
            next.gesture = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Vec<DashboardItem> {
        vec![
            DashboardItem::new("a", 0, 0, 6, 2),
            DashboardItem::new("b", 6, 0, 6, 2),
            DashboardItem::new("c", 0, 2, 12, 2),
        ]
    }

    fn input() -> DashboardInput {
        DashboardInput { default_items: board(), ..DashboardInput::default() }
    }

    fn cells(items: &[DashboardItem]) -> Vec<(u32, u32, u32, u32)> {
        items.iter().map(|i| (i.x, i.y, i.w, i.h)).collect()
    }

    fn run(state: &DashboardState, event: DashboardEvent) -> hydroxy_statechart::Step<DashboardState, DashboardCommand> {
        DashboardLayoutMachine::transition(state, event, &input()).resolve(state)
    }

    #[test]
    fn place_pushes_colliders_down_and_closes_gaps() {
        // Dropping `a` onto `b`'s spot pushes `b` below it; `c` follows.
        let out = place(&board(), "a", 6, 0, 6, 2, 12);
        assert_eq!(cells(&out), [(6, 0, 6, 2), (6, 2, 6, 2), (0, 4, 12, 2)]);
    }

    #[test]
    fn place_clamps_into_the_grid() {
        let out = place(&board(), "a", -5, -5, 99, 0, 12);
        assert_eq!(cells(&out)[0], (0, 0, 12, 1));
        let out = place(&board(), "b", 50, 0, 6, 2, 12);
        assert_eq!(cells(&out)[1].0, 6);
        assert_eq!(place(&board(), "nope", 0, 0, 1, 1, 12), board());
    }

    #[test]
    fn place_floats_items_up() {
        let items = vec![DashboardItem::new("a", 0, 5, 4, 1), DashboardItem::new("b", 4, 9, 4, 1)];
        let out = place(&items, "a", 0, 5, 4, 1, 12);
        assert_eq!(cells(&out), [(0, 5, 4, 1), (4, 0, 4, 1)]);
    }

    #[test]
    fn keyboard_move_previews_then_drops() {
        let s0 = DashboardLayoutMachine::initial(&input());
        let grabbed = run(&s0, DashboardEvent::Grab("a".into()));
        assert!(matches!(grabbed.commands[0], DashboardCommand::Announce(Announcement::Grabbed(_))));
        let moved = run(&grabbed.state, DashboardEvent::Nudge { dx: 1, dy: 0, resize: false });
        assert_eq!(moved.state.preview(12)[0].x, 1);
        assert_eq!(moved.state.items[0].x, 0, "preview only until dropped");
        let dropped = run(&moved.state, DashboardEvent::Drop);
        assert_eq!(dropped.state.items[0].x, 1);
        assert!(dropped.state.gesture.is_none());
        assert!(matches!(dropped.commands[0], DashboardCommand::LayoutChange(_)));
    }

    #[test]
    fn edge_nudges_do_not_build_up_debt() {
        let s0 = DashboardLayoutMachine::initial(&input());
        let mut s = run(&s0, DashboardEvent::Grab("b".into())).state;
        for _ in 0..3 {
            s = run(&s, DashboardEvent::Nudge { dx: 1, dy: 0, resize: false }).state;
        }
        let s = run(&s, DashboardEvent::Nudge { dx: -1, dy: 0, resize: false }).state;
        assert_eq!(s.preview(12)[1].x, 5);
    }

    #[test]
    fn keyboard_resize_and_cancel() {
        let s0 = DashboardLayoutMachine::initial(&input());
        let s = run(&s0, DashboardEvent::Grab("a".into())).state;
        let s = run(&s, DashboardEvent::Nudge { dx: -1, dy: 1, resize: true }).state;
        assert_eq!((s.preview(12)[0].w, s.preview(12)[0].h), (5, 3));
        let cancelled = run(&s, DashboardEvent::Cancel);
        assert_eq!(cancelled.state.items, board());
        assert!(matches!(cancelled.commands[0], DashboardCommand::Announce(Announcement::Cancelled(_))));
    }

    #[test]
    fn pointer_drag_resizes_from_the_grip() {
        let s0 = DashboardLayoutMachine::initial(&input());
        let s = run(&s0, DashboardEvent::DragStart { id: "a".into(), resize: true }).state;
        let s = run(&s, DashboardEvent::DragTo { dx: 2, dy: 1 }).state;
        let done = run(&s, DashboardEvent::Drop);
        assert_eq!(cells(&done.state.items)[0], (0, 0, 8, 3));
    }

    #[test]
    fn controlled_layout_waits_for_the_owner() {
        let inp = DashboardInput { items: Some(board()), ..DashboardInput::default() };
        let s0 = DashboardLayoutMachine::initial(&inp);
        let s = DashboardLayoutMachine::transition(&s0, DashboardEvent::DragStart { id: "a".into(), resize: false }, &inp).resolve(&s0).state;
        let s = DashboardLayoutMachine::transition(&s, DashboardEvent::DragTo { dx: 1, dy: 0 }, &inp).resolve(&s).state;
        let done = DashboardLayoutMachine::transition(&s, DashboardEvent::Drop, &inp).resolve(&s);
        assert_eq!(done.state.items, board());
        assert!(matches!(&done.commands[0], DashboardCommand::LayoutChange(items) if items[0].x == 1));
    }

    #[test]
    fn disabled_ignores_everything() {
        let inp = DashboardInput { disabled: true, ..input() };
        let s0 = DashboardLayoutMachine::initial(&inp);
        let step = DashboardLayoutMachine::transition(&s0, DashboardEvent::Grab("a".into()), &inp).resolve(&s0);
        assert!(step.state.gesture.is_none());
    }
}
