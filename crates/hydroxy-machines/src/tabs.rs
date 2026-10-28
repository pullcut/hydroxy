//! Tabs owns selection and roving focus. Automatic activation selects on
//! keyboard navigation; manual activation moves focus only. Orientation is a
//! connector concern: it maps physical keys to `Direction`.

use hydroxy_statechart::{Machine, Transition};

pub use crate::ChangeReason;
pub use crate::list::Direction;
use crate::list::next_index;

/// One tab, identified by its `value`.
#[derive(Debug, Clone, PartialEq)]
pub struct TabsItem {
    /// Unique key. Ties the trigger to its content and is what selection reports.
    pub value: String,
    /// Skipped by keyboard navigation and ignored on click.
    pub disabled: bool,
}

impl TabsItem {
    /// An enabled tab.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), disabled: false }
    }

    /// Mark this tab disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// When keyboard navigation selects a tab.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Activation {
    /// Arrow keys move focus and select.
    #[default]
    Automatic,
    /// Arrow keys move focus only; Enter or Space (a click) selects.
    Manual,
}

/// Props for [`TabsMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct TabsInput {
    /// The tabs, in display order.
    pub tabs: Vec<TabsItem>,
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial value.
    pub default_value: Option<String>,
    /// See [`Activation`].
    pub activation: Activation,
    /// Wrap focus from the last tab to the first and back.
    pub loop_focus: bool,
}

impl TabsInput {
    /// Uncontrolled, automatic activation, focus loops.
    #[must_use]
    pub fn new(tabs: Vec<TabsItem>) -> Self {
        Self { tabs, value: None, default_value: None, activation: Activation::Automatic, loop_focus: true }
    }
}

/// What [`TabsMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TabsState {
    /// The selected tab.
    pub value: Option<String>,
    /// The tab whose trigger last had focus.
    pub focused: Option<String>,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`TabsMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum TabsEvent {
    /// Select this tab.
    Select(String, ChangeReason),
    /// Arrow key: move focus one enabled tab along.
    Move(Direction),
    /// Focus the first enabled tab.
    Home,
    /// Focus the last enabled tab.
    End,
    /// A trigger received focus (e.g. by Tab key or click).
    Focus(String),
}

/// Side effects from [`TabsMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum TabsCommand {
    /// Move DOM focus to this tab's trigger.
    FocusTrigger(String),
    /// Tell the owner the value changed (or should, when controlled).
    ValueChange {
        /// The newly selected tab.
        value: String,
        /// What caused the change.
        reason: ChangeReason,
    },
}

/// Selection and roving focus for a set of tabs.
#[derive(Debug, Clone, Copy)]
pub struct TabsMachine;

type Step = Transition<TabsState, TabsCommand>;

impl Machine for TabsMachine {
    type Input = TabsInput;
    type State = TabsState;
    type Event = TabsEvent;
    type Command = TabsCommand;

    fn initial(input: &TabsInput) -> TabsState {
        TabsState {
            value: input.value.clone().or_else(|| input.default_value.clone()),
            focused: None,
            controlled: input.value.is_some(),
        }
    }

    fn transition(state: &TabsState, event: TabsEvent, input: &TabsInput) -> Step {
        let enabled: Vec<&str> = input.tabs.iter().filter(|t| !t.disabled).map(|t| t.value.as_str()).collect();
        let is_enabled = |v: &str| enabled.contains(&v);

        match event {
            TabsEvent::Select(value, reason) => {
                if !is_enabled(&value) || state.value.as_deref() == Some(value.as_str()) {
                    return Transition::stay();
                }
                select(state.clone(), value, reason)
            }
            TabsEvent::Move(direction) => {
                if enabled.is_empty() {
                    return Transition::stay();
                }
                let start = state.focused.as_deref().or(state.value.as_deref());
                let current = start.and_then(|v| enabled.iter().position(|e| *e == v));
                let next = next_index(current, enabled.len(), direction, input.loop_focus);
                move_focus(state, input, enabled[next])
            }
            TabsEvent::Home => match enabled.first() {
                Some(first) => move_focus(state, input, first),
                None => Transition::stay(),
            },
            TabsEvent::End => match enabled.last() {
                Some(last) => move_focus(state, input, last),
                None => Transition::stay(),
            },
            TabsEvent::Focus(value) => {
                let mut next = state.clone();
                if is_enabled(&value) {
                    next.focused = Some(value);
                }
                Transition::to(next)
            }
        }
    }

    fn sync(state: &TabsState, input: &TabsInput) -> Option<TabsState> {
        let mut next = state.clone();
        if state.controlled {
            next.value.clone_from(&input.value);
        }
        hydroxy_statechart::changed(state, next)
    }
}

/// Commit (when uncontrolled) and always request the change.
fn select(mut state: TabsState, value: String, reason: ChangeReason) -> Step {
    if !state.controlled {
        state.value = Some(value.clone());
    }
    Transition::with(state, vec![TabsCommand::ValueChange { value, reason }])
}

/// MOVE/HOME/END: focus `target`; automatic activation also selects it.
fn move_focus(state: &TabsState, input: &TabsInput, target: &str) -> Step {
    let automatic = input.activation == Activation::Automatic;
    let focus_changed = state.focused.as_deref().or(state.value.as_deref()) != Some(target);
    let selection_changed = automatic && state.value.as_deref() != Some(target);
    if !focus_changed && !selection_changed {
        return Transition::stay();
    }

    let mut next = state.clone();
    next.focused = Some(target.to_string());
    let mut commands = vec![TabsCommand::FocusTrigger(target.to_string())];
    if selection_changed {
        let step = select(next, target.to_string(), ChangeReason::Keyboard);
        commands.extend(step.commands);
        return Transition { state: step.state, commands };
    }
    Transition::with(next, commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> TabsInput {
        let mut input = TabsInput::new(vec![TabsItem::new("a"), TabsItem::new("b").disabled(), TabsItem::new("c")]);
        input.default_value = Some("a".into());
        input
    }

    fn run(state: &TabsState, event: TabsEvent, input: &TabsInput) -> hydroxy_statechart::Step<TabsState, TabsCommand> {
        TabsMachine::transition(state, event, input).resolve(state)
    }

    #[test]
    fn select_commits_when_uncontrolled() {
        let input = input();
        let step = run(&TabsMachine::initial(&input), TabsEvent::Select("c".into(), ChangeReason::Pointer), &input);
        assert_eq!(step.state.value.as_deref(), Some("c"));
        assert_eq!(step.commands, vec![TabsCommand::ValueChange { value: "c".into(), reason: ChangeReason::Pointer }]);
    }

    #[test]
    fn select_ignores_disabled_and_current() {
        let input = input();
        let state = TabsMachine::initial(&input);
        assert!(run(&state, TabsEvent::Select("b".into(), ChangeReason::Pointer), &input).commands.is_empty());
        assert!(run(&state, TabsEvent::Select("a".into(), ChangeReason::Pointer), &input).commands.is_empty());
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = input();
        input.value = Some("a".into());
        let state = TabsMachine::initial(&input);
        let step = run(&state, TabsEvent::Select("c".into(), ChangeReason::Pointer), &input);
        assert_eq!(step.state.value.as_deref(), Some("a"));
        assert_eq!(step.commands.len(), 1);

        input.value = Some("c".into());
        assert_eq!(hydroxy_statechart::sync_or_keep::<TabsMachine>(&step.state, &input).value.as_deref(), Some("c"));
    }

    #[test]
    fn move_skips_disabled_and_selects_in_automatic_mode() {
        let input = input();
        let step = run(&TabsMachine::initial(&input), TabsEvent::Move(Direction::Next), &input);
        assert_eq!(step.state.focused.as_deref(), Some("c"));
        assert_eq!(step.state.value.as_deref(), Some("c"));
        assert_eq!(step.commands[0], TabsCommand::FocusTrigger("c".into()));
    }

    #[test]
    fn move_wraps_when_looping() {
        let input = input();
        let state = TabsState { value: Some("c".into()), focused: Some("c".into()), controlled: false };
        assert_eq!(run(&state, TabsEvent::Move(Direction::Next), &input).state.focused.as_deref(), Some("a"));
    }

    #[test]
    fn move_stops_at_ends_without_loop() {
        let mut input = input();
        input.loop_focus = false;
        let state = TabsState { value: Some("c".into()), focused: Some("c".into()), controlled: false };
        assert!(run(&state, TabsEvent::Move(Direction::Next), &input).commands.is_empty());
    }

    #[test]
    fn move_enters_at_first_or_last_when_nothing_selected() {
        let input = TabsInput::new(vec![TabsItem::new("a"), TabsItem::new("b")]);
        let state = TabsMachine::initial(&input);
        assert_eq!(run(&state, TabsEvent::Move(Direction::Next), &input).state.focused.as_deref(), Some("a"));
        assert_eq!(run(&state, TabsEvent::Move(Direction::Prev), &input).state.focused.as_deref(), Some("b"));
    }

    #[test]
    fn manual_activation_moves_focus_only() {
        let mut input = input();
        input.activation = Activation::Manual;
        let step = run(&TabsMachine::initial(&input), TabsEvent::End, &input);
        assert_eq!(step.state.focused.as_deref(), Some("c"));
        assert_eq!(step.state.value.as_deref(), Some("a"));
        assert_eq!(step.commands, vec![TabsCommand::FocusTrigger("c".into())]);
    }
}
