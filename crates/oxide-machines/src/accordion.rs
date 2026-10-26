//! Accordion owns which items are open and which trigger has focus.
//! Single mode keeps at most one item open; multiple mode toggles each item
//! on its own. Arrow keys move focus between triggers and never toggle.

use oxide_statechart::{Machine, Transition};

pub use crate::list::Direction;
use crate::list::next_index;

/// One accordion item, identified by its `value`.
#[derive(Debug, Clone, PartialEq)]
pub struct AccordionEntry {
    /// Unique key. Ties the trigger to its content and is what the value reports.
    pub value: String,
    /// Skipped by keyboard navigation and ignored on click.
    pub disabled: bool,
}

impl AccordionEntry {
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

/// Props for [`AccordionMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct AccordionInput {
    /// The items, in display order.
    pub items: Vec<AccordionEntry>,
    /// Allow several items open at once.
    pub multiple: bool,
    /// Controlled open items. `Some` at start fixes controlled mode.
    pub value: Option<Vec<String>>,
    /// Uncontrolled initially open items.
    pub default_value: Vec<String>,
    /// In single mode, whether the open item can be closed again.
    pub collapsible: bool,
}

impl AccordionInput {
    /// Uncontrolled, single, collapsible, nothing open.
    #[must_use]
    pub fn new(items: Vec<AccordionEntry>) -> Self {
        Self { items, multiple: false, value: None, default_value: Vec::new(), collapsible: true }
    }
}

/// What [`AccordionMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AccordionState {
    /// Open items, in the order they were opened.
    pub value: Vec<String>,
    /// The item whose trigger last had focus.
    pub focused: Option<String>,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`AccordionMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum AccordionEvent {
    /// Open or close this item (click, Enter or Space).
    Toggle(String),
    /// Arrow key: move focus one enabled trigger along. Always wraps.
    Move(Direction),
    /// Focus the first enabled trigger.
    Home,
    /// Focus the last enabled trigger.
    End,
    /// A trigger received focus.
    Focus(String),
}

/// Side effects from [`AccordionMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum AccordionCommand {
    /// Move DOM focus to this item's trigger.
    FocusTrigger(String),
    /// Tell the owner the open items changed (or should, when controlled).
    ValueChange(Vec<String>),
}

/// Open state and trigger focus for a set of accordion items.
#[derive(Debug, Clone, Copy)]
pub struct AccordionMachine;

type Step = Transition<AccordionState, AccordionCommand>;

impl Machine for AccordionMachine {
    type Input = AccordionInput;
    type State = AccordionState;
    type Event = AccordionEvent;
    type Command = AccordionCommand;

    fn initial(input: &AccordionInput) -> AccordionState {
        AccordionState {
            value: input.value.clone().unwrap_or_else(|| input.default_value.clone()),
            focused: None,
            controlled: input.value.is_some(),
        }
    }

    fn transition(state: &AccordionState, event: AccordionEvent, input: &AccordionInput) -> Step {
        let enabled: Vec<&str> = input.items.iter().filter(|i| !i.disabled).map(|i| i.value.as_str()).collect();

        match event {
            AccordionEvent::Toggle(value) => {
                if !enabled.contains(&value.as_str()) {
                    return Transition::stay(state.clone());
                }
                let open = state.value.contains(&value);
                let next_value = match (input.multiple, open) {
                    (true, true) => state.value.iter().filter(|v| **v != value).cloned().collect(),
                    (true, false) => state.value.iter().cloned().chain([value]).collect(),
                    (false, true) if !input.collapsible => return Transition::stay(state.clone()),
                    (false, true) => Vec::new(),
                    (false, false) => vec![value],
                };
                let mut next = state.clone();
                if !state.controlled {
                    next.value.clone_from(&next_value);
                }
                Transition { state: next, commands: vec![AccordionCommand::ValueChange(next_value)] }
            }
            AccordionEvent::Move(direction) => {
                if enabled.is_empty() {
                    return Transition::stay(state.clone());
                }
                let current = state.focused.as_deref().and_then(|v| enabled.iter().position(|e| *e == v));
                move_focus(state, enabled[next_index(current, enabled.len(), direction, true)])
            }
            AccordionEvent::Home => match enabled.first() {
                Some(first) => move_focus(state, first),
                None => Transition::stay(state.clone()),
            },
            AccordionEvent::End => match enabled.last() {
                Some(last) => move_focus(state, last),
                None => Transition::stay(state.clone()),
            },
            AccordionEvent::Focus(value) => {
                let mut next = state.clone();
                if enabled.contains(&value.as_str()) {
                    next.focused = Some(value);
                }
                Transition::stay(next)
            }
        }
    }

    fn sync(state: &AccordionState, input: &AccordionInput) -> AccordionState {
        let mut next = state.clone();
        if state.controlled {
            next.value = input.value.clone().unwrap_or_default();
        }
        next
    }
}

/// MOVE/HOME/END: focus `target`. Navigation never toggles.
fn move_focus(state: &AccordionState, target: &str) -> Step {
    if state.focused.as_deref() == Some(target) {
        return Transition::stay(state.clone());
    }
    let mut next = state.clone();
    next.focused = Some(target.to_string());
    Transition { state: next, commands: vec![AccordionCommand::FocusTrigger(target.to_string())] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> AccordionInput {
        AccordionInput::new(vec![
            AccordionEntry::new("a"),
            AccordionEntry::new("b").disabled(),
            AccordionEntry::new("c"),
        ])
    }

    fn run(state: &AccordionState, event: AccordionEvent, input: &AccordionInput) -> Step {
        AccordionMachine::transition(state, event, input)
    }

    fn toggle(v: &str) -> AccordionEvent {
        AccordionEvent::Toggle(v.into())
    }

    #[test]
    fn single_replaces_and_closes_when_collapsible() {
        let input = input();
        let s = run(&AccordionMachine::initial(&input), toggle("a"), &input).state;
        let step = run(&s, toggle("c"), &input);
        assert_eq!(step.state.value, vec!["c"]);
        assert_eq!(step.commands, vec![AccordionCommand::ValueChange(vec!["c".into()])]);
        assert!(run(&step.state, toggle("c"), &input).state.value.is_empty());
    }

    #[test]
    fn single_not_collapsible_keeps_open_item() {
        let mut input = input();
        input.collapsible = false;
        input.default_value = vec!["a".into()];
        let step = run(&AccordionMachine::initial(&input), toggle("a"), &input);
        assert_eq!(step.state.value, vec!["a"]);
        assert!(step.commands.is_empty());
    }

    #[test]
    fn multiple_adds_and_removes_in_order() {
        let mut input = input();
        input.multiple = true;
        let s = run(&AccordionMachine::initial(&input), toggle("c"), &input).state;
        let s = run(&s, toggle("a"), &input).state;
        assert_eq!(s.value, vec!["c", "a"]);
        assert_eq!(run(&s, toggle("c"), &input).state.value, vec!["a"]);
    }

    #[test]
    fn disabled_item_ignored() {
        let input = input();
        assert!(run(&AccordionMachine::initial(&input), toggle("b"), &input).commands.is_empty());
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = input();
        input.value = Some(vec![]);
        let step = run(&AccordionMachine::initial(&input), toggle("a"), &input);
        assert!(step.state.value.is_empty());
        assert_eq!(step.commands.len(), 1);
        input.value = Some(vec!["a".into()]);
        assert_eq!(AccordionMachine::sync(&step.state, &input).value, vec!["a"]);
    }

    #[test]
    fn arrows_skip_disabled_wrap_and_never_toggle() {
        let input = input();
        let state = AccordionState { focused: Some("a".into()), ..Default::default() };
        let step = run(&state, AccordionEvent::Move(Direction::Next), &input);
        assert_eq!(step.state.focused.as_deref(), Some("c"));
        assert_eq!(step.commands, vec![AccordionCommand::FocusTrigger("c".into())]);
        assert!(step.state.value.is_empty());
        assert_eq!(run(&step.state, AccordionEvent::Move(Direction::Next), &input).state.focused.as_deref(), Some("a"));
        assert_eq!(run(&state, AccordionEvent::End, &input).state.focused.as_deref(), Some("c"));
    }
}
