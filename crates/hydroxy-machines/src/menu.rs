//! Menu owns open state and the highlighted item. Focus follows the highlight:
//! the adapter focuses the highlighted item while the menu is open. Dismissal
//! (Esc, outside click) is native through the Popover API; the adapter reports
//! it as [`MenuEvent::Close`].

use hydroxy_statechart::{Machine, Transition};

pub use crate::list::Direction;
use crate::list::next_index;

/// One menu item, identified by its `value`.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuEntry {
    /// Unique key. Reported by `on_select`.
    pub value: String,
    /// Skipped by keyboard navigation and ignored on click.
    pub disabled: bool,
}

impl MenuEntry {
    /// An enabled item.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), disabled: false }
    }

    /// Mark this item disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Props for [`MenuMachine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MenuInput {
    /// The items, in display order.
    pub items: Vec<MenuEntry>,
    /// Controlled open state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled initial open state.
    pub default_open: bool,
}

/// What [`MenuMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MenuState {
    /// Whether the menu is showing.
    pub open: bool,
    /// The highlighted (and, while open, focused) item.
    pub highlighted: Option<String>,
    /// Fixed at start: the owner holds `open` and the machine only requests changes.
    pub controlled: bool,
}

/// Which item an opening menu highlights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Highlight {
    /// The first enabled item (click, Enter, Space, `ArrowDown`).
    First,
    /// The last enabled item (`ArrowUp`).
    Last,
}

/// Inputs to [`MenuMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum MenuEvent {
    /// Trigger clicked: open on the first item, or close.
    Toggle,
    /// Open and highlight an end. No-op while open.
    Open(Highlight),
    /// Close. `restore_focus` sends focus back to the trigger (Esc does; Tab and outside clicks don't).
    Close {
        /// Emit [`MenuCommand::FocusTrigger`].
        restore_focus: bool,
    },
    /// Arrow key: highlight the next enabled item, wrapping.
    Move(Direction),
    /// Highlight the first enabled item.
    Home,
    /// Highlight the last enabled item.
    End,
    /// Pointer moved over, or focus landed on, this item.
    Highlight(String),
    /// Activate this item, then close and return focus to the trigger.
    Select(String),
}

/// Side effects from [`MenuMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuCommand {
    /// Tell the owner the open state changed (or should, when controlled).
    OpenChange(bool),
    /// An item was activated.
    Select(String),
    /// Move DOM focus back to the trigger.
    FocusTrigger,
}

/// Open state and highlight for a menu of actions.
#[derive(Debug, Clone, Copy)]
pub struct MenuMachine;

type Step = Transition<MenuState, MenuCommand>;

fn enabled(input: &MenuInput) -> Vec<&str> {
    input.items.iter().filter(|i| !i.disabled).map(|i| i.value.as_str()).collect()
}

/// Commit (when uncontrolled) and always request the change.
fn set_open(state: &mut MenuState, open: bool, commands: &mut Vec<MenuCommand>) {
    if !state.controlled {
        state.open = open;
    }
    commands.push(MenuCommand::OpenChange(open));
}

fn open(state: &MenuState, input: &MenuInput, at: Highlight) -> Step {
    let items = enabled(input);
    let mut next = state.clone();
    next.highlighted = match at {
        Highlight::First => items.first(),
        Highlight::Last => items.last(),
    }
    .map(|v| (*v).to_string());
    let mut commands = Vec::new();
    set_open(&mut next, true, &mut commands);
    Transition::update(state, next, commands)
}

fn close(state: &MenuState, restore_focus: bool) -> Step {
    let mut next = state.clone();
    let mut commands = Vec::new();
    set_open(&mut next, false, &mut commands);
    if restore_focus {
        commands.push(MenuCommand::FocusTrigger);
    }
    Transition::update(state, next, commands)
}

fn highlight(state: &MenuState, value: &str) -> Step {
    let mut next = state.clone();
    next.highlighted = Some(value.to_string());
    Transition::update(state, next, Vec::new())
}

impl Machine for MenuMachine {
    type Input = MenuInput;
    type State = MenuState;
    type Event = MenuEvent;
    type Command = MenuCommand;

    fn initial(input: &MenuInput) -> MenuState {
        MenuState {
            open: input.open.unwrap_or(input.default_open),
            highlighted: None,
            controlled: input.open.is_some(),
        }
    }

    fn transition(state: &MenuState, event: MenuEvent, input: &MenuInput) -> Step {
        let items = enabled(input);
        match event {
            MenuEvent::Toggle if state.open => close(state, false),
            MenuEvent::Toggle => open(state, input, Highlight::First),
            MenuEvent::Open(at) if !state.open => open(state, input, at),
            MenuEvent::Close { restore_focus } if state.open => close(state, restore_focus),
            MenuEvent::Move(direction) if state.open && !items.is_empty() => {
                let current = state.highlighted.as_deref().and_then(|h| items.iter().position(|v| *v == h));
                highlight(state, items[next_index(current, items.len(), direction, true)])
            }
            MenuEvent::Home if state.open && !items.is_empty() => highlight(state, items[0]),
            MenuEvent::End if state.open && !items.is_empty() => highlight(state, items[items.len() - 1]),
            MenuEvent::Highlight(value) if input.items.iter().any(|i| !i.disabled && i.value == value) => {
                highlight(state, &value)
            }
            MenuEvent::Select(value) if state.open && items.contains(&value.as_str()) => {
                let step = close(state, true);
                let mut commands = vec![MenuCommand::Select(value)];
                commands.extend(step.commands);
                Transition { state: step.state, commands }
            }
            MenuEvent::Open(..)
            | MenuEvent::Close { .. }
            | MenuEvent::Move(..)
            | MenuEvent::Home
            | MenuEvent::End
            | MenuEvent::Highlight(..)
            | MenuEvent::Select(..) => Transition::stay(),
        }
    }

    fn sync(state: &MenuState, input: &MenuInput) -> Option<MenuState> {
        let mut next = state.clone();
        if state.controlled {
            next.open = input.open.unwrap_or(false);
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn input() -> MenuInput {
        MenuInput {
            items: vec![MenuEntry::new("rename"), MenuEntry::new("archive").disabled(), MenuEntry::new("delete")],
            ..MenuInput::default()
        }
    }

    #[test]
    fn toggle_opens_on_first_and_closes() {
        let input = input();
        let step = hydroxy_statechart::step::<MenuMachine>(&MenuMachine::initial(&input), MenuEvent::Toggle, &input);
        assert!(step.state.open);
        assert_eq!(step.state.highlighted.as_deref(), Some("rename"));
        assert_eq!(step.commands, vec![MenuCommand::OpenChange(true)]);
        let step = hydroxy_statechart::step::<MenuMachine>(&step.state, MenuEvent::Toggle, &input);
        assert!(!step.state.open);
        assert_eq!(step.commands, vec![MenuCommand::OpenChange(false)]);
    }

    #[test]
    fn arrow_up_opens_on_last() {
        let input = input();
        let step = hydroxy_statechart::step::<MenuMachine>(
            &MenuMachine::initial(&input),
            MenuEvent::Open(Highlight::Last),
            &input,
        );
        assert_eq!(step.state.highlighted.as_deref(), Some("delete"));
    }

    #[test]
    fn move_skips_disabled_and_wraps() {
        let input = input();
        let open = step::<MenuMachine>(&MenuMachine::initial(&input), MenuEvent::Toggle, &input).state;
        let next = step::<MenuMachine>(&open, MenuEvent::Move(Direction::Next), &input).state;
        assert_eq!(next.highlighted.as_deref(), Some("delete"));
        let wrapped = step::<MenuMachine>(&next, MenuEvent::Move(Direction::Next), &input).state;
        assert_eq!(wrapped.highlighted.as_deref(), Some("rename"));
    }

    #[test]
    fn select_reports_closes_and_returns_focus() {
        let input = input();
        let open =
            hydroxy_statechart::step::<MenuMachine>(&MenuMachine::initial(&input), MenuEvent::Toggle, &input).state;
        let step = hydroxy_statechart::step::<MenuMachine>(&open, MenuEvent::Select("delete".into()), &input);
        assert!(!step.state.open);
        assert_eq!(
            step.commands,
            vec![MenuCommand::Select("delete".into()), MenuCommand::OpenChange(false), MenuCommand::FocusTrigger]
        );
        assert!(
            hydroxy_statechart::step::<MenuMachine>(&open, MenuEvent::Select("archive".into()), &input)
                .commands
                .is_empty()
        );
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = input();
        input.open = Some(false);
        let state = MenuMachine::initial(&input);
        let step = hydroxy_statechart::step::<MenuMachine>(&state, MenuEvent::Toggle, &input);
        assert!(!step.state.open);
        assert_eq!(step.commands, vec![MenuCommand::OpenChange(true)]);
        input.open = Some(true);
        assert!(hydroxy_statechart::sync_or_keep::<MenuMachine>(&step.state, &input).open);
    }

    #[test]
    fn close_while_closed_is_a_no_op() {
        let input = input();
        let step = hydroxy_statechart::step::<MenuMachine>(
            &MenuMachine::initial(&input),
            MenuEvent::Close { restore_focus: true },
            &input,
        );
        assert!(step.commands.is_empty());
    }
}
