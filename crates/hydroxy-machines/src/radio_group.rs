//! Radio group owns one selected value and roving focus across its items.
//! Arrow keys move focus and select, skipping disabled items. A disabled
//! group ignores every event.

use hydroxy_statechart::{Controllable, Machine, Transition};

pub use crate::ChangeReason;

pub use crate::list::Direction;
use crate::list::next_index;

/// One option, identified by its `value`.
#[derive(Debug, Clone, PartialEq)]
pub struct RadioItem {
    /// Unique key. What selection reports and what a form submits.
    pub value: String,
    /// Skipped by arrow keys and ignored on click.
    pub disabled: bool,
}

impl RadioItem {
    /// An enabled option.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), disabled: false }
    }

    /// Mark this option disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Props for [`RadioGroupMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct RadioGroupInput {
    /// The options, in display order.
    pub items: Vec<RadioItem>,
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial value.
    pub default_value: Option<String>,
    /// Disable the whole group.
    pub disabled: bool,
    /// Wrap focus from the last option to the first and back.
    pub loop_focus: bool,
}

impl RadioGroupInput {
    /// Uncontrolled, enabled, focus loops.
    #[must_use]
    pub fn new(items: Vec<RadioItem>) -> Self {
        Self { items, value: None, default_value: None, disabled: false, loop_focus: true }
    }

    /// Whether `value` names an option that can be selected.
    #[must_use]
    pub fn is_enabled(&self, value: &str) -> bool {
        !self.disabled && self.items.iter().any(|i| i.value == value && !i.disabled)
    }
}

/// What [`RadioGroupMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadioGroupState {
    /// The selected option. Controlled when the owner holds it: the machine then only requests
    /// changes.
    pub value: Controllable<Option<String>>,
    /// The option that last had focus.
    pub focused: Option<String>,
}

/// Inputs to [`RadioGroupMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum RadioGroupEvent {
    /// Select this option.
    Select(String, ChangeReason),
    /// Arrow key: focus and select the next enabled option.
    Move(Direction),
    /// An option received focus.
    Focus(String),
}

/// Side effects from [`RadioGroupMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum RadioGroupCommand {
    /// Move DOM focus to this option.
    FocusItem(String),
    /// Tell the owner the value changed (or should, when controlled).
    ValueChange {
        /// The newly selected option.
        value: String,
        /// What caused the change.
        reason: ChangeReason,
    },
}

/// Selection and roving focus for a group of radios.
#[derive(Debug, Clone, Copy)]
pub struct RadioGroupMachine;

type Step = Transition<RadioGroupState, RadioGroupCommand>;

impl Machine for RadioGroupMachine {
    type Input = RadioGroupInput;
    type State = RadioGroupState;
    type Event = RadioGroupEvent;
    type Command = RadioGroupCommand;

    fn initial(input: &RadioGroupInput) -> RadioGroupState {
        RadioGroupState {
            value: Controllable::new(
                input.value.clone().or_else(|| input.default_value.clone()),
                input.value.is_some(),
            ),
            focused: None,
        }
    }

    fn transition(state: &RadioGroupState, event: RadioGroupEvent, input: &RadioGroupInput) -> Step {
        match event {
            RadioGroupEvent::Select(value, reason) => {
                if !input.is_enabled(&value) || state.value.value.as_deref() == Some(value.as_str()) {
                    return Transition::stay();
                }
                select(state.clone(), value, reason)
            }
            RadioGroupEvent::Move(direction) => {
                let enabled: Vec<&str> = input.items.iter().filter(|i| !i.disabled).map(|i| i.value.as_str()).collect();
                if input.disabled || enabled.is_empty() {
                    return Transition::stay();
                }
                let start = state.focused.as_deref().or(state.value.value.as_deref());
                let current = start.and_then(|v| enabled.iter().position(|e| *e == v));
                let target = enabled[next_index(current, enabled.len(), direction, input.loop_focus)];
                if current.map(|i| enabled[i]) == Some(target) {
                    return Transition::stay();
                }
                let mut next = state.clone();
                next.focused = Some(target.to_string());
                let mut commands = vec![RadioGroupCommand::FocusItem(target.to_string())];
                if state.value.value.as_deref() != Some(target) {
                    let step = select(next, target.to_string(), ChangeReason::Keyboard);
                    commands.extend(step.commands);
                    return Transition { state: step.state, commands };
                }
                Transition::update(state, next, commands)
            }
            RadioGroupEvent::Focus(value) => {
                let mut next = state.clone();
                if input.is_enabled(&value) {
                    next.focused = Some(value);
                }
                Transition::update(state, next, Vec::new())
            }
        }
    }

    fn sync(state: &RadioGroupState, input: &RadioGroupInput) -> Option<RadioGroupState> {
        let mut next = state.clone();
        next.value.sync(Some(input.value.clone()));
        hydroxy_statechart::changed(state, next)
    }
}

/// Commit (when uncontrolled) and always request the change.
fn select(mut state: RadioGroupState, value: String, reason: ChangeReason) -> Step {
    state.value.set(Some(value.clone()));
    Transition::with(state, vec![RadioGroupCommand::ValueChange { value, reason }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn input() -> RadioGroupInput {
        let mut input =
            RadioGroupInput::new(vec![RadioItem::new("a"), RadioItem::new("b").disabled(), RadioItem::new("c")]);
        input.default_value = Some("a".into());
        input
    }

    #[test]
    fn select_commits_when_uncontrolled() {
        let input = input();
        let step = hydroxy_statechart::step::<RadioGroupMachine>(
            &RadioGroupMachine::initial(&input),
            RadioGroupEvent::Select("c".into(), ChangeReason::Pointer),
            &input,
        );
        assert_eq!(step.state.value.value.as_deref(), Some("c"));
        assert_eq!(
            step.commands,
            vec![RadioGroupCommand::ValueChange { value: "c".into(), reason: ChangeReason::Pointer }]
        );
    }

    #[test]
    fn select_ignores_disabled_item_current_value_and_disabled_group() {
        let mut input = input();
        let state = RadioGroupMachine::initial(&input);
        let pointer = |v: &str| RadioGroupEvent::Select(v.into(), ChangeReason::Pointer);
        assert!(step::<RadioGroupMachine>(&state, pointer("b"), &input).commands.is_empty());
        assert!(step::<RadioGroupMachine>(&state, pointer("a"), &input).commands.is_empty());
        input.disabled = true;
        assert!(step::<RadioGroupMachine>(&state, pointer("c"), &input).commands.is_empty());
        assert!(step::<RadioGroupMachine>(&state, RadioGroupEvent::Move(Direction::Next), &input).commands.is_empty());
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = input();
        input.value = Some("a".into());
        let state = RadioGroupMachine::initial(&input);
        let step = hydroxy_statechart::step::<RadioGroupMachine>(
            &state,
            RadioGroupEvent::Select("c".into(), ChangeReason::Pointer),
            &input,
        );
        assert_eq!(step.state.value.value.as_deref(), Some("a"));
        assert_eq!(step.commands.len(), 1);

        input.value = Some("c".into());
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<RadioGroupMachine>(&step.state, &input).value.value.as_deref(),
            Some("c")
        );
    }

    #[test]
    fn arrows_skip_disabled_focus_and_select() {
        let input = input();
        let step = hydroxy_statechart::step::<RadioGroupMachine>(
            &RadioGroupMachine::initial(&input),
            RadioGroupEvent::Move(Direction::Next),
            &input,
        );
        assert_eq!(step.state.value.value.as_deref(), Some("c"));
        assert_eq!(
            step.commands,
            vec![
                RadioGroupCommand::FocusItem("c".into()),
                RadioGroupCommand::ValueChange { value: "c".into(), reason: ChangeReason::Keyboard },
            ]
        );
        // Wraps back to the first option.
        let wrapped =
            hydroxy_statechart::step::<RadioGroupMachine>(&step.state, RadioGroupEvent::Move(Direction::Next), &input);
        assert_eq!(wrapped.state.value.value.as_deref(), Some("a"));
    }

    #[test]
    fn arrows_stop_at_ends_without_loop() {
        let mut input = input();
        input.loop_focus = false;
        let state = RadioGroupState { value: Controllable::new(Some("c".into()), false), focused: Some("c".into()) };
        assert!(step::<RadioGroupMachine>(&state, RadioGroupEvent::Move(Direction::Next), &input).commands.is_empty());
    }
}
