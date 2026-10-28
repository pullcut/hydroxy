//! Menubar: a row of menus where one at a time is open. This machine owns
//! which menu is open and which trigger is the tab stop. Each menu's items are
//! a [`crate::menu::MenuMachine`], driven by the adapter with a controlled
//! open flag. While a menu is open, moving over or arrowing to another
//! trigger switches to that menu.

use hydroxy_statechart::{Machine, Transition};

pub use crate::list::Direction;
use crate::list::next_index;

/// One menu in the bar, identified by its `value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenubarEntry {
    /// Unique key.
    pub value: String,
    /// Skipped by arrow keys, and can't open.
    pub disabled: bool,
}

impl MenubarEntry {
    /// An enabled menu.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), disabled: false }
    }

    /// Mark this menu disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Props for [`MenubarMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MenubarInput {
    /// The menus, in display order.
    pub menus: Vec<MenubarEntry>,
    /// Controlled open menu (`Some(None)` is "none open"). `Some` at start fixes controlled mode.
    pub open: Option<Option<String>>,
    /// Uncontrolled initial open menu.
    pub default_open: Option<String>,
}

/// What [`MenubarMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MenubarState {
    /// The open menu, if any.
    pub open: Option<String>,
    /// The menu whose trigger is the tab stop.
    pub active: Option<String>,
    /// Fixed at start: the owner holds `open` and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`MenubarMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenubarEvent {
    /// Trigger clicked: open this menu, or close it if it is the open one.
    Toggle(String),
    /// Open this menu (`ArrowDown` on its trigger).
    Open(String),
    /// Pointer moved onto this trigger. Switches menus only while one is open.
    Hover(String),
    /// This menu closed itself (Esc, a selection or an outside press).
    Dismiss(String),
    /// This trigger took focus.
    Focus(String),
    /// Left or Right arrow: the next or previous menu, wrapping.
    Move(Direction),
    /// Home: the first menu.
    Home,
    /// End: the last menu.
    End,
}

/// Side effects from [`MenubarMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenubarCommand {
    /// Tell the owner the open menu changed (or should, when controlled).
    OpenChange(Option<String>),
    /// Move DOM focus to this menu's trigger.
    FocusTrigger(String),
}

/// Open menu and tab stop for one menubar.
#[derive(Debug, Clone, Copy)]
pub struct MenubarMachine;

type Step = Transition<MenubarState, MenubarCommand>;

fn enabled(input: &MenubarInput) -> Vec<&str> {
    input.menus.iter().filter(|m| !m.disabled).map(|m| m.value.as_str()).collect()
}

/// Commit (when uncontrolled) and always request the change.
fn request_open(state: &MenubarState, open: Option<String>, active: Option<String>) -> Step {
    let mut next = state.clone();
    if active.is_some() {
        next.active = active;
    }
    let mut commands = Vec::new();
    if open != state.open {
        if !state.controlled {
            next.open.clone_from(&open);
        }
        commands.push(MenubarCommand::OpenChange(open));
    }
    Transition::with(next, commands)
}

/// Go to `target`: switch the open menu if one is open, else just move the tab stop and focus.
fn go(state: &MenubarState, target: &str) -> Step {
    if state.open.is_some() {
        request_open(state, Some(target.to_string()), Some(target.to_string()))
    } else {
        let mut next = state.clone();
        next.active = Some(target.to_string());
        Transition::with(next, vec![MenubarCommand::FocusTrigger(target.to_string())])
    }
}

impl Machine for MenubarMachine {
    type Input = MenubarInput;
    type State = MenubarState;
    type Event = MenubarEvent;
    type Command = MenubarCommand;

    fn initial(input: &MenubarInput) -> MenubarState {
        let open = input.open.clone().unwrap_or_else(|| input.default_open.clone());
        let active = open.clone().or_else(|| enabled(input).first().map(|v| (*v).to_string()));
        MenubarState { open, active, controlled: input.open.is_some() }
    }

    fn transition(state: &MenubarState, event: MenubarEvent, input: &MenubarInput) -> Step {
        let items = enabled(input);
        let known = |v: &str| items.contains(&v);
        match event {
            MenubarEvent::Toggle(v) if known(&v) => {
                let open = if state.open.as_deref() == Some(v.as_str()) { None } else { Some(v.clone()) };
                request_open(state, open, Some(v))
            }
            MenubarEvent::Open(v) if known(&v) => request_open(state, Some(v.clone()), Some(v)),
            MenubarEvent::Hover(v) if known(&v) && state.open.is_some() => {
                request_open(state, Some(v.clone()), Some(v))
            }
            MenubarEvent::Dismiss(v) if state.open.as_deref() == Some(v.as_str()) => request_open(state, None, None),
            MenubarEvent::Focus(v) if known(&v) => {
                let mut next = state.clone();
                next.active = Some(v);
                Transition::to(next)
            }
            MenubarEvent::Move(direction) if !items.is_empty() => {
                let from = state.open.as_ref().or(state.active.as_ref());
                let current = from.and_then(|f| items.iter().position(|v| *v == f.as_str()));
                go(state, items[next_index(current, items.len(), direction, true)])
            }
            MenubarEvent::Home if !items.is_empty() => go(state, items[0]),
            MenubarEvent::End if !items.is_empty() => go(state, items[items.len() - 1]),
            _ => Transition::stay(),
        }
    }

    fn sync(state: &MenubarState, input: &MenubarInput) -> Option<MenubarState> {
        let mut next = state.clone();
        if state.controlled {
            next.open = input.open.clone().flatten();
            if next.open.is_some() {
                next.active.clone_from(&next.open);
            }
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> MenubarInput {
        MenubarInput {
            menus: vec![MenubarEntry::new("file"), MenubarEntry::new("edit").disabled(), MenubarEntry::new("view")],
            ..Default::default()
        }
    }

    fn run(
        state: &MenubarState,
        event: MenubarEvent,
        input: &MenubarInput,
    ) -> hydroxy_statechart::Step<MenubarState, MenubarCommand> {
        MenubarMachine::transition(state, event, input).resolve(state)
    }

    fn opened(input: &MenubarInput, menu: &str) -> MenubarState {
        run(&MenubarMachine::initial(input), MenubarEvent::Toggle(menu.into()), input).state
    }

    #[test]
    fn first_enabled_menu_is_the_tab_stop() {
        assert_eq!(MenubarMachine::initial(&input()).active.as_deref(), Some("file"));
    }

    #[test]
    fn toggle_opens_and_closes() {
        let input = input();
        let open = run(&MenubarMachine::initial(&input), MenubarEvent::Toggle("file".into()), &input);
        assert_eq!(open.state.open.as_deref(), Some("file"));
        assert_eq!(open.commands, vec![MenubarCommand::OpenChange(Some("file".into()))]);
        let closed = run(&open.state, MenubarEvent::Toggle("file".into()), &input);
        assert_eq!(closed.state.open, None);
        assert_eq!(closed.commands, vec![MenubarCommand::OpenChange(None)]);
    }

    #[test]
    fn disabled_menus_never_open() {
        let input = input();
        let state = MenubarMachine::initial(&input);
        assert!(run(&state, MenubarEvent::Toggle("edit".into()), &input).commands.is_empty());
        assert!(run(&state, MenubarEvent::Open("edit".into()), &input).commands.is_empty());
    }

    #[test]
    fn hover_switches_only_while_open() {
        let input = input();
        let closed = MenubarMachine::initial(&input);
        assert!(run(&closed, MenubarEvent::Hover("view".into()), &input).commands.is_empty());
        let step = run(&opened(&input, "file"), MenubarEvent::Hover("view".into()), &input);
        assert_eq!(step.state.open.as_deref(), Some("view"));
        assert_eq!(step.state.active.as_deref(), Some("view"));
    }

    #[test]
    fn arrows_move_focus_when_closed_and_switch_when_open() {
        let input = input();
        let closed = MenubarMachine::initial(&input);
        let step = run(&closed, MenubarEvent::Move(Direction::Next), &input);
        assert_eq!(step.commands, vec![MenubarCommand::FocusTrigger("view".into())]);
        assert_eq!(step.state.active.as_deref(), Some("view"));
        let wrapped = run(&step.state, MenubarEvent::Move(Direction::Next), &input);
        assert_eq!(wrapped.state.active.as_deref(), Some("file"));

        let switched = run(&opened(&input, "file"), MenubarEvent::Move(Direction::Prev), &input);
        assert_eq!(switched.state.open.as_deref(), Some("view"));
        assert_eq!(switched.commands, vec![MenubarCommand::OpenChange(Some("view".into()))]);
    }

    #[test]
    fn home_and_end_jump_to_the_ends() {
        let input = input();
        let state = MenubarMachine::initial(&input);
        assert_eq!(run(&state, MenubarEvent::End, &input).state.active.as_deref(), Some("view"));
        assert_eq!(run(&state, MenubarEvent::Home, &input).state.active.as_deref(), Some("file"));
    }

    #[test]
    fn dismiss_closes_only_the_open_menu() {
        let input = input();
        let state = opened(&input, "file");
        assert!(run(&state, MenubarEvent::Dismiss("view".into()), &input).commands.is_empty());
        assert_eq!(run(&state, MenubarEvent::Dismiss("file".into()), &input).state.open, None);
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = MenubarInput { open: Some(None), ..input() };
        let state = MenubarMachine::initial(&input);
        let step = run(&state, MenubarEvent::Toggle("file".into()), &input);
        assert_eq!(step.state.open, None);
        assert_eq!(step.commands, vec![MenubarCommand::OpenChange(Some("file".into()))]);
        input.open = Some(Some("file".into()));
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<MenubarMachine>(&step.state, &input).open.as_deref(),
            Some("file")
        );
    }
}
