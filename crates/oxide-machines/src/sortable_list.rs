//! Sortable list: one list of items reordered by pointer drag or keyboard
//! (Space, arrows, Space, Esc). It is the kanban machine run over a single
//! column, so the move and drag logic lives in one place. This layer adds the
//! list-shaped input and a `Reorder` command that carries the whole new order.

use oxide_statechart::{Machine, Transition};

use crate::kanban::{
    KanbanCard, KanbanColumn, KanbanCommand, KanbanEvent, KanbanInput, KanbanMachine, KanbanMove,
    KanbanState, apply_move,
};

pub use crate::kanban::{Announcement, KanbanEvent as SortableEvent, MoveDirection};

/// One item. The adapter renders `title` unless the owner supplies its own view.
pub type SortItem = KanbanCard;

/// The id of the single column the list lives in.
pub const COLUMN: &str = "list";

/// Props for [`SortableMachine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SortableInput {
    /// Controlled order. `Some` at start fixes controlled mode.
    pub items: Option<Vec<SortItem>>,
    /// Uncontrolled initial order.
    pub default_items: Vec<SortItem>,
    /// Block every move.
    pub disabled: bool,
}

impl SortableInput {
    fn column(items: &[SortItem]) -> Vec<KanbanColumn> {
        vec![KanbanColumn::new(COLUMN, "List", items.to_vec())]
    }

    fn kanban(&self) -> KanbanInput {
        KanbanInput {
            columns: self.items.as_deref().map(Self::column),
            default_columns: Self::column(&self.default_items),
            disabled: self.disabled,
        }
    }
}

/// What [`SortableMachine`] remembers between events. Use the helpers on
/// [`SortableState`] rather than the kanban fields.
pub type SortableState = KanbanState;

/// The items in order, as committed.
#[must_use]
pub fn items(state: &SortableState) -> Vec<SortItem> {
    state.columns.first().map(|c| c.cards.clone()).unwrap_or_default()
}

/// The items to render. During a keyboard move the item shows where it would land.
#[must_use]
pub fn preview(state: &SortableState) -> Vec<SortItem> {
    state.preview().into_iter().next().map(|c| c.cards).unwrap_or_default()
}

/// `items` with the item `id` moved to `index` among the others.
#[must_use]
pub fn moved(items: &[SortItem], id: &str, index: usize) -> Vec<SortItem> {
    let mut columns = SortableInput::column(items);
    apply_move(&mut columns, &KanbanMove { card: id.into(), from: COLUMN.into(), to: COLUMN.into(), index });
    columns.remove(0).cards
}

/// Side effects from [`SortableMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum SortableCommand {
    /// The order changed (or should, when controlled). Carries the full new order.
    Reorder(Vec<SortItem>),
    /// Move DOM focus to this item.
    FocusItem(String),
    /// Say this in the live region. `column` is meaningless here; see the headless wording.
    Announce(Announcement),
}

/// Order and drag state for a sortable list.
#[derive(Debug, Clone, Copy)]
pub struct SortableMachine;

type Step = Transition<SortableState, SortableCommand>;

impl Machine for SortableMachine {
    type Input = SortableInput;
    type State = SortableState;
    type Event = KanbanEvent;
    type Command = SortableCommand;

    fn initial(input: &SortableInput) -> SortableState {
        KanbanMachine::initial(&input.kanban())
    }

    fn transition(state: &SortableState, event: KanbanEvent, input: &SortableInput) -> Step {
        let step = KanbanMachine::transition(state, event, &input.kanban());
        // The kanban step may already have applied the move, so build the new
        // order from the state we started with.
        let before = items(state);
        let commands = step
            .commands
            .into_iter()
            .map(|c| match c {
                KanbanCommand::Move(mv) => SortableCommand::Reorder(moved(&before, &mv.card, mv.index)),
                KanbanCommand::FocusCard(id) => SortableCommand::FocusItem(id),
                KanbanCommand::Announce(a) => SortableCommand::Announce(a),
            })
            .collect();
        Transition { state: step.state, commands }
    }

    fn sync(state: &SortableState, input: &SortableInput) -> Option<SortableState> {
        KanbanMachine::sync(state, &input.kanban())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> Vec<SortItem> {
        ["a", "b", "c"].iter().map(|i| SortItem::new(*i, i.to_uppercase())).collect()
    }

    fn ids(items: &[SortItem]) -> Vec<&str> {
        items.iter().map(|i| i.id.as_str()).collect()
    }

    fn input() -> SortableInput {
        SortableInput { default_items: list(), ..SortableInput::default() }
    }

    fn run(state: &SortableState, event: KanbanEvent, input: &SortableInput) -> oxide_statechart::Step<SortableState, SortableCommand> {
        SortableMachine::transition(state, event, input).resolve(state)
    }

    #[test]
    fn moved_reorders() {
        assert_eq!(ids(&moved(&list(), "a", 2)), ["b", "c", "a"]);
        assert_eq!(ids(&moved(&list(), "c", 0)), ["c", "a", "b"]);
        assert_eq!(ids(&moved(&list(), "zzz", 0)), ["a", "b", "c"]);
    }

    #[test]
    fn keyboard_move_previews_then_reorders() {
        let i = input();
        let mut s = SortableMachine::initial(&i);
        s = run(&s, KanbanEvent::Grab("a".into()), &i).state;
        s = run(&s, KanbanEvent::Move(MoveDirection::Down), &i).state;
        assert_eq!(ids(&preview(&s)), ["b", "a", "c"]);
        assert_eq!(ids(&items(&s)), ["a", "b", "c"]);
        // Left and right have nowhere to go in a single list.
        assert_eq!(run(&s, KanbanEvent::Move(MoveDirection::Left), &i).state, s);
        let step = run(&s, KanbanEvent::Drop, &i);
        assert_eq!(ids(&items(&step.state)), ["b", "a", "c"]);
        assert!(step.commands.contains(&SortableCommand::Reorder(moved(&list(), "a", 1))));
    }

    #[test]
    fn controlled_reports_the_order_but_keeps_its_own() {
        let i = SortableInput { items: Some(list()), ..SortableInput::default() };
        let mut s = SortableMachine::initial(&i);
        s = run(&s, KanbanEvent::Grab("c".into()), &i).state;
        s = run(&s, KanbanEvent::Move(MoveDirection::Up), &i).state;
        let step = run(&s, KanbanEvent::Drop, &i);
        assert_eq!(ids(&items(&step.state)), ["a", "b", "c"]);
        assert!(step.commands.contains(&SortableCommand::Reorder(moved(&list(), "c", 1))));
    }

    #[test]
    fn escape_cancels() {
        let i = input();
        let mut s = SortableMachine::initial(&i);
        s = run(&s, KanbanEvent::Grab("a".into()), &i).state;
        s = run(&s, KanbanEvent::Move(MoveDirection::Down), &i).state;
        let step = run(&s, KanbanEvent::Cancel, &i);
        assert_eq!(ids(&preview(&step.state)), ["a", "b", "c"]);
        assert!(!step.commands.iter().any(|c| matches!(c, SortableCommand::Reorder(_))));
    }

    #[test]
    fn pointer_drag_drops_before_a_target() {
        let i = input();
        let mut s = SortableMachine::initial(&i);
        s = run(&s, KanbanEvent::DragStart("a".into()), &i).state;
        s = run(&s, KanbanEvent::DragOver { column: COLUMN.into(), before: None }, &i).state;
        let step = run(&s, KanbanEvent::Drop, &i);
        assert_eq!(ids(&items(&step.state)), ["b", "c", "a"]);
    }
}
