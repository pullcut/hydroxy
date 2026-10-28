//! Kanban owns the card layout and the card being moved. A move starts with a
//! pointer drag or a keyboard pick-up (Space), follows the pointer or the arrow
//! keys, and ends with a drop or a cancel. Only a drop that changes something
//! reports a [`KanbanMove`].

use hydroxy_statechart::{Machine, Transition};

/// One card. The adapter renders `title` unless the owner supplies its own view.
#[derive(Debug, Clone, PartialEq)]
pub struct KanbanCard {
    /// Unique key across the board.
    pub id: String,
    /// Visible text, also used in announcements.
    pub title: String,
}

impl KanbanCard {
    /// A card.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self { id: id.into(), title: title.into() }
    }
}

/// One column and its cards, top to bottom.
#[derive(Debug, Clone, PartialEq)]
pub struct KanbanColumn {
    /// Unique key.
    pub id: String,
    /// Visible heading, also the column's accessible name.
    pub title: String,
    /// The cards, in order.
    pub cards: Vec<KanbanCard>,
}

impl KanbanColumn {
    /// A column.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>, cards: Vec<KanbanCard>) -> Self {
        Self { id: id.into(), title: title.into(), cards }
    }
}

/// A finished move: card `card` left column `from` for position `index` in `to`.
/// `index` counts the destination's cards without the moved one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KanbanMove {
    /// The card's id.
    pub card: String,
    /// The source column's id.
    pub from: String,
    /// The destination column's id. May equal `from` for a reorder.
    pub to: String,
    /// Position in the destination.
    pub index: usize,
}

/// Apply `mv` to `columns`. Unknown cards or columns leave them unchanged.
/// Controlled owners call this from `on_move`.
pub fn apply_move(columns: &mut [KanbanColumn], mv: &KanbanMove) {
    let Some(to) = columns.iter().position(|c| c.id == mv.to) else { return };
    let Some((ci, ki)) = locate(columns, &mv.card) else { return };
    let card = columns[ci].cards.remove(ki);
    let dest = &mut columns[to].cards;
    dest.insert(mv.index.min(dest.len()), card);
}

/// (column index, card index) of a card.
fn locate(columns: &[KanbanColumn], card: &str) -> Option<(usize, usize)> {
    columns.iter().enumerate().find_map(|(ci, c)| c.cards.iter().position(|k| k.id == card).map(|ki| (ci, ki)))
}

/// Props for [`KanbanMachine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KanbanInput {
    /// Controlled layout. `Some` at start fixes controlled mode.
    pub columns: Option<Vec<KanbanColumn>>,
    /// Uncontrolled initial layout.
    pub default_columns: Vec<KanbanColumn>,
    /// Block every move.
    pub disabled: bool,
}

/// The card in flight and where it would land.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grab {
    /// The card's id.
    pub card: String,
    /// The column it came from.
    pub from: String,
    /// Where it would land: column id.
    pub column: String,
    /// Where it would land: index among that column's other cards.
    pub index: usize,
    /// Picked up with Space rather than a pointer drag.
    pub keyboard: bool,
}

/// What [`KanbanMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KanbanState {
    /// The committed layout.
    pub columns: Vec<KanbanColumn>,
    /// The card being moved, if any.
    pub grab: Option<Grab>,
    /// Fixed at start: the owner holds the layout.
    pub controlled: bool,
}

impl KanbanState {
    /// The layout to render. During a keyboard move the card shows where it
    /// would land; pointer drags leave the layout alone and mark the target instead.
    #[must_use]
    pub fn preview(&self) -> Vec<KanbanColumn> {
        let mut columns = self.columns.clone();
        if let Some(g) = self.grab.as_ref().filter(|g| g.keyboard) {
            apply_move(&mut columns, &g.pending());
        }
        columns
    }
}

impl Grab {
    fn pending(&self) -> KanbanMove {
        KanbanMove { card: self.card.clone(), from: self.from.clone(), to: self.column.clone(), index: self.index }
    }
}

/// Which way an arrow key moves a picked-up card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    /// Up one place.
    Up,
    /// Down one place.
    Down,
    /// To the previous column.
    Left,
    /// To the next column.
    Right,
}

/// Inputs to [`KanbanMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum KanbanEvent {
    /// Space on a card: pick it up.
    Grab(String),
    /// Arrow key while a card is picked up.
    Move(MoveDirection),
    /// A pointer drag started on this card.
    DragStart(String),
    /// The pointer drag is over `column`, before card `before` or at the end.
    DragOver {
        /// The column under the pointer.
        column: String,
        /// The card under the pointer. `None` means after the last card.
        before: Option<String>,
    },
    /// Space while carrying, or a pointer drop.
    Drop,
    /// Esc, or a pointer drag that ended outside the board.
    Cancel,
}

/// What a screen reader should hear. Format with your own words or the headless defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Announcement {
    /// Picked up `card` in `column` at `position` of `count` (1-based).
    Grabbed {
        /// Card title.
        card: String,
        /// Column title.
        column: String,
        /// 1-based position.
        position: usize,
        /// Cards in the column.
        count: usize,
    },
    /// Carried to `position` of `count` in `column`.
    Moved {
        /// Card title.
        card: String,
        /// Column title.
        column: String,
        /// 1-based position.
        position: usize,
        /// Cards in the column, counting this one.
        count: usize,
    },
    /// Dropped in `column` at `position`.
    Dropped {
        /// Card title.
        card: String,
        /// Column title.
        column: String,
        /// 1-based position.
        position: usize,
    },
    /// Put back where it started.
    Cancelled {
        /// Card title.
        card: String,
    },
}

/// Side effects from [`KanbanMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum KanbanCommand {
    /// A card moved (or should, when controlled).
    Move(KanbanMove),
    /// Move DOM focus to this card.
    FocusCard(String),
    /// Say this in the live region.
    Announce(Announcement),
}

/// Card layout and drag state for a board.
#[derive(Debug, Clone, Copy)]
pub struct KanbanMachine;

type Step = Transition<KanbanState, KanbanCommand>;

fn title(columns: &[KanbanColumn], column: &str) -> String {
    columns.iter().find(|c| c.id == column).map(|c| c.title.clone()).unwrap_or_default()
}

fn card_title(columns: &[KanbanColumn], card: &str) -> String {
    locate(columns, card).map(|(ci, ki)| columns[ci].cards[ki].title.clone()).unwrap_or_default()
}

/// Cards in `column` other than `card`: the range a landing index can take.
fn others(columns: &[KanbanColumn], column: &str, card: &str) -> usize {
    columns.iter().find(|c| c.id == column).map_or(0, |c| c.cards.iter().filter(|k| k.id != card).count())
}

fn pick_up(state: &KanbanState, card: &str, keyboard: bool) -> Step {
    let Some((ci, ki)) = locate(&state.columns, card) else { return Transition::stay() };
    let column = &state.columns[ci];
    let mut next = state.clone();
    next.grab =
        Some(Grab { card: card.to_string(), from: column.id.clone(), column: column.id.clone(), index: ki, keyboard });
    let commands = if keyboard {
        vec![KanbanCommand::Announce(Announcement::Grabbed {
            card: column.cards[ki].title.clone(),
            column: column.title.clone(),
            position: ki + 1,
            count: column.cards.len(),
        })]
    } else {
        Vec::new()
    };
    Transition::with(next, commands)
}

impl Machine for KanbanMachine {
    type Input = KanbanInput;
    type State = KanbanState;
    type Event = KanbanEvent;
    type Command = KanbanCommand;

    fn initial(input: &KanbanInput) -> KanbanState {
        KanbanState {
            columns: input.columns.clone().unwrap_or_else(|| input.default_columns.clone()),
            grab: None,
            controlled: input.columns.is_some(),
        }
    }

    fn transition(state: &KanbanState, event: KanbanEvent, input: &KanbanInput) -> Step {
        let stay = || Transition::stay();
        if input.disabled {
            return stay();
        }
        let cols = &state.columns;
        match (event, &state.grab) {
            (KanbanEvent::Grab(card), None) => pick_up(state, &card, true),
            (KanbanEvent::DragStart(card), _) => pick_up(state, &card, false),
            (KanbanEvent::Move(direction), Some(g)) if g.keyboard => {
                let mut grab = g.clone();
                match direction {
                    MoveDirection::Up => grab.index = grab.index.saturating_sub(1),
                    MoveDirection::Down => grab.index = (grab.index + 1).min(others(cols, &g.column, &g.card)),
                    MoveDirection::Left | MoveDirection::Right => {
                        let Some(at) = cols.iter().position(|c| c.id == g.column) else { return stay() };
                        let to = if direction == MoveDirection::Left { at.checked_sub(1) } else { Some(at + 1) };
                        let Some(column) = to.and_then(|i| cols.get(i)) else { return stay() };
                        grab.column.clone_from(&column.id);
                        grab.index = grab.index.min(others(cols, &column.id, &g.card));
                    }
                }
                if grab == *g {
                    return stay();
                }
                let announce = Announcement::Moved {
                    card: card_title(cols, &g.card),
                    column: title(cols, &grab.column),
                    position: grab.index + 1,
                    count: others(cols, &grab.column, &g.card) + 1,
                };
                let card = g.card.clone();
                let mut next = state.clone();
                next.grab = Some(grab);
                // The card re-renders in its new spot, so focus has to follow it.
                Transition::with(next, vec![KanbanCommand::FocusCard(card), KanbanCommand::Announce(announce)])
            }
            (KanbanEvent::DragOver { column, before }, Some(g)) if !g.keyboard => {
                let Some(col) = cols.iter().find(|c| c.id == column) else { return stay() };
                let rest: Vec<&str> = col.cards.iter().filter(|k| k.id != g.card).map(|k| k.id.as_str()).collect();
                let index = match before.as_deref() {
                    // Over the dragged card itself: its own slot.
                    Some(b) if b == g.card => locate(cols, b).map_or(0, |(_, ki)| ki),
                    Some(b) => rest.iter().position(|k| *k == b).unwrap_or(rest.len()),
                    None => rest.len(),
                };
                if g.column == column && g.index == index {
                    return stay();
                }
                let mut next = state.clone();
                next.grab = Some(Grab { column, index, ..g.clone() });
                Transition::to(next)
            }
            (KanbanEvent::Drop, Some(g)) => {
                let mv = g.pending();
                let mut next = state.clone();
                next.grab = None;
                let mut commands = Vec::new();
                let unchanged = locate(cols, &g.card).is_some_and(|(ci, ki)| cols[ci].id == mv.to && ki == mv.index);
                if !unchanged {
                    if !state.controlled {
                        apply_move(&mut next.columns, &mv);
                    }
                    commands.push(KanbanCommand::Move(mv.clone()));
                }
                if g.keyboard {
                    commands.push(KanbanCommand::FocusCard(g.card.clone()));
                    commands.push(KanbanCommand::Announce(Announcement::Dropped {
                        card: card_title(cols, &g.card),
                        column: title(cols, &mv.to),
                        position: mv.index + 1,
                    }));
                }
                Transition::with(next, commands)
            }
            (KanbanEvent::Cancel, Some(g)) => {
                let mut next = state.clone();
                next.grab = None;
                let commands = if g.keyboard {
                    vec![
                        KanbanCommand::FocusCard(g.card.clone()),
                        KanbanCommand::Announce(Announcement::Cancelled { card: card_title(cols, &g.card) }),
                    ]
                } else {
                    Vec::new()
                };
                Transition::with(next, commands)
            }
            _ => stay(),
        }
    }

    fn sync(state: &KanbanState, input: &KanbanInput) -> Option<KanbanState> {
        let mut next = state.clone();
        if state.controlled {
            next.columns = input.columns.clone().unwrap_or_default();
        }
        if next.grab.as_ref().is_some_and(|g| locate(&next.columns, &g.card).is_none()) || input.disabled {
            next.grab = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Vec<KanbanColumn> {
        vec![
            KanbanColumn::new("todo", "To do", vec![KanbanCard::new("a", "A"), KanbanCard::new("b", "B")]),
            KanbanColumn::new("doing", "Doing", vec![KanbanCard::new("c", "C")]),
            KanbanColumn::new("done", "Done", vec![]),
        ]
    }

    fn input() -> KanbanInput {
        KanbanInput { default_columns: board(), ..KanbanInput::default() }
    }

    fn layout(columns: &[KanbanColumn]) -> Vec<Vec<&str>> {
        columns.iter().map(|c| c.cards.iter().map(|k| k.id.as_str()).collect()).collect()
    }

    fn run(state: &KanbanState, event: KanbanEvent, input: &KanbanInput) -> hydroxy_statechart::Step<KanbanState, KanbanCommand> {
        KanbanMachine::transition(state, event, input).resolve(state)
    }

    fn mv(card: &str, from: &str, to: &str, index: usize) -> KanbanMove {
        KanbanMove { card: card.into(), from: from.into(), to: to.into(), index }
    }

    #[test]
    fn apply_move_reorders_and_crosses_columns() {
        let mut cols = board();
        apply_move(&mut cols, &mv("a", "todo", "todo", 1));
        assert_eq!(layout(&cols), [vec!["b", "a"], vec!["c"], vec![]]);
        apply_move(&mut cols, &mv("a", "todo", "doing", 0));
        assert_eq!(layout(&cols), [vec!["b"], vec!["a", "c"], vec![]]);
        apply_move(&mut cols, &mv("b", "todo", "done", 99));
        assert_eq!(layout(&cols), [vec![], vec!["a", "c"], vec!["b"]]);
        apply_move(&mut cols, &mv("zzz", "todo", "done", 0));
        apply_move(&mut cols, &mv("a", "doing", "nope", 0));
        assert_eq!(layout(&cols), [vec![], vec!["a", "c"], vec!["b"]]);
    }

    #[test]
    fn keyboard_pick_up_move_drop() {
        let input = input();
        let s = KanbanMachine::initial(&input);
        let step = run(&s, KanbanEvent::Grab("a".into()), &input);
        assert!(matches!(
            &step.commands[..],
            [KanbanCommand::Announce(Announcement::Grabbed { position: 1, count: 2, .. })]
        ));
        let s = run(&step.state, KanbanEvent::Move(MoveDirection::Down), &input).state;
        assert_eq!(layout(&s.preview()), [vec!["b", "a"], vec!["c"], vec![]]);
        // Already last among the other cards: Down does nothing.
        assert!(run(&s, KanbanEvent::Move(MoveDirection::Down), &input).commands.is_empty());
        let step = run(&s, KanbanEvent::Move(MoveDirection::Right), &input);
        assert_eq!(layout(&step.state.preview()), [vec!["b"], vec!["c", "a"], vec![]]);
        assert_eq!(step.commands[0], KanbanCommand::FocusCard("a".into()));
        let step = run(&step.state, KanbanEvent::Drop, &input);
        assert_eq!(step.commands[0], KanbanCommand::Move(mv("a", "todo", "doing", 1)));
        assert_eq!(layout(&step.state.columns), [vec!["b"], vec!["c", "a"], vec![]]);
        assert!(step.state.grab.is_none());
    }

    #[test]
    fn left_right_stop_at_the_edges_and_clamp_index() {
        let input = input();
        let s = run(&KanbanMachine::initial(&input), KanbanEvent::Grab("b".into()), &input).state;
        assert!(run(&s, KanbanEvent::Move(MoveDirection::Left), &input).commands.is_empty());
        let s = run(&s, KanbanEvent::Move(MoveDirection::Right), &input).state;
        let s = run(&s, KanbanEvent::Move(MoveDirection::Right), &input).state;
        assert_eq!(s.grab.as_ref().map(|g| (g.column.as_str(), g.index)), Some(("done", 0)));
        assert!(run(&s, KanbanEvent::Move(MoveDirection::Right), &input).commands.is_empty());
    }

    #[test]
    fn cancel_restores_and_drop_in_place_reports_nothing() {
        let input = input();
        let s = run(&KanbanMachine::initial(&input), KanbanEvent::Grab("a".into()), &input).state;
        let moved = run(&s, KanbanEvent::Move(MoveDirection::Right), &input).state;
        let step = run(&moved, KanbanEvent::Cancel, &input);
        assert_eq!(layout(&step.state.preview()), layout(&board()));
        assert!(matches!(step.commands[1], KanbanCommand::Announce(Announcement::Cancelled { .. })));
        let step = run(&s, KanbanEvent::Drop, &input);
        assert!(!step.commands.iter().any(|c| matches!(c, KanbanCommand::Move(_))));
    }

    #[test]
    fn pointer_drag_targets_before_a_card_or_the_end() {
        let input = input();
        let s = run(&KanbanMachine::initial(&input), KanbanEvent::DragStart("a".into()), &input).state;
        let over = |s: &KanbanState, column: &str, before: Option<&str>| {
            run(s, KanbanEvent::DragOver { column: column.into(), before: before.map(Into::into) }, &input).state
        };
        let s = over(&s, "doing", Some("c"));
        assert_eq!(s.grab.as_ref().map(|g| g.index), Some(0));
        // Pointer drags don't reshuffle the preview.
        assert_eq!(layout(&s.preview()), layout(&board()));
        let s = over(&s, "todo", None);
        assert_eq!(s.grab.as_ref().map(|g| (g.column.as_str(), g.index)), Some(("todo", 1)));
        let step = run(&s, KanbanEvent::Drop, &input);
        assert_eq!(step.commands, [KanbanCommand::Move(mv("a", "todo", "todo", 1))]);
        assert_eq!(layout(&step.state.columns), [vec!["b", "a"], vec!["c"], vec![]]);
        // Over its own slot, then dropped: nothing changes.
        let s = run(&step.state, KanbanEvent::DragStart("a".into()), &input).state;
        let s = over(&s, "todo", Some("a"));
        assert!(run(&s, KanbanEvent::Drop, &input).commands.is_empty());
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = KanbanInput { columns: Some(board()), ..KanbanInput::default() };
        let s = run(&KanbanMachine::initial(&input), KanbanEvent::DragStart("c".into()), &input).state;
        let s = run(&s, KanbanEvent::DragOver { column: "done".into(), before: None }, &input).state;
        let step = run(&s, KanbanEvent::Drop, &input);
        assert_eq!(step.commands, [KanbanCommand::Move(mv("c", "doing", "done", 0))]);
        assert_eq!(layout(&step.state.columns), layout(&board()));
        let mut next = board();
        apply_move(&mut next, &mv("c", "doing", "done", 0));
        input.columns = Some(next);
        assert_eq!(layout(&hydroxy_statechart::sync_or_keep::<KanbanMachine>(&step.state, &input).columns), [vec!["a", "b"], vec![], vec!["c"]]);
    }

    #[test]
    fn disabled_blocks_everything() {
        let input = KanbanInput { disabled: true, ..input() };
        assert!(run(&KanbanMachine::initial(&input), KanbanEvent::Grab("a".into()), &input).state.grab.is_none());
    }
}
