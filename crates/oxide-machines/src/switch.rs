//! Switch owns one on/off value. Uncontrolled presses commit and report;
//! controlled presses only request, and the owner's value arrives via `sync`.

use oxide_statechart::{Machine, Transition};

pub use crate::ChangeReason;

/// Props for [`SwitchMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwitchInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub checked: Option<bool>,
    /// Uncontrolled initial value.
    pub default_checked: bool,
    /// Presses are ignored.
    pub disabled: bool,
}

/// What [`SwitchMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwitchState {
    /// Whether the switch is on.
    pub checked: bool,
    /// Fixed at start: the owner holds the value and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`SwitchMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchEvent {
    /// Flip the switch.
    Toggle(ChangeReason),
}

/// Side effects from [`SwitchMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchCommand {
    /// Tell the owner the value changed (or should, when controlled).
    CheckedChange {
        /// The new (or proposed) value.
        checked: bool,
        /// What caused the change.
        reason: ChangeReason,
    },
}

/// A single on/off binding.
#[derive(Debug, Clone, Copy)]
pub struct SwitchMachine;

impl Machine for SwitchMachine {
    type Input = SwitchInput;
    type State = SwitchState;
    type Event = SwitchEvent;
    type Command = SwitchCommand;

    fn initial(input: &SwitchInput) -> SwitchState {
        SwitchState { checked: input.checked.unwrap_or(input.default_checked), controlled: input.checked.is_some() }
    }

    fn transition(
        state: &SwitchState,
        event: SwitchEvent,
        input: &SwitchInput,
    ) -> Transition<SwitchState, SwitchCommand> {
        let SwitchEvent::Toggle(reason) = event;
        if input.disabled {
            return Transition::stay();
        }
        let checked = !state.checked;
        let mut next = state.clone();
        if !state.controlled {
            next.checked = checked;
        }
        Transition::with(next, vec![SwitchCommand::CheckedChange { checked, reason }])
    }

    fn sync(state: &SwitchState, input: &SwitchInput) -> Option<SwitchState> {
        let mut next = state.clone();
        if let (true, Some(checked)) = (state.controlled, input.checked) {
            next.checked = checked;
        }
        oxide_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toggle(state: &SwitchState, input: &SwitchInput) -> oxide_statechart::Step<SwitchState, SwitchCommand> {
        SwitchMachine::transition(state, SwitchEvent::Toggle(ChangeReason::Pointer), input).resolve(state)
    }

    #[test]
    fn uncontrolled_commits_and_reports() {
        let input = SwitchInput::default();
        let step = toggle(&SwitchMachine::initial(&input), &input);
        assert!(step.state.checked);
        assert_eq!(step.commands, vec![SwitchCommand::CheckedChange { checked: true, reason: ChangeReason::Pointer }]);
        assert!(!toggle(&step.state, &input).state.checked);
    }

    #[test]
    fn controlled_false_is_controlled_and_waits_for_sync() {
        let mut input = SwitchInput { checked: Some(false), ..SwitchInput::default() };
        let state = SwitchMachine::initial(&input);
        assert!(state.controlled);

        let step = toggle(&state, &input);
        assert!(!step.state.checked);
        assert_eq!(step.commands, vec![SwitchCommand::CheckedChange { checked: true, reason: ChangeReason::Pointer }]);

        input.checked = Some(true);
        assert!(oxide_statechart::sync_or_keep::<SwitchMachine>(&step.state, &input).checked);
    }

    #[test]
    fn disabled_ignores_presses() {
        let input = SwitchInput { disabled: true, ..SwitchInput::default() };
        let state = SwitchMachine::initial(&input);
        let step = toggle(&state, &input);
        assert_eq!(step.state, state);
        assert!(step.commands.is_empty());
    }

    #[test]
    fn sync_leaves_uncontrolled_alone() {
        let input = SwitchInput { default_checked: true, ..SwitchInput::default() };
        let state = SwitchMachine::initial(&input);
        assert_eq!(oxide_statechart::sync_or_keep::<SwitchMachine>(&state, &SwitchInput::default()), state);
    }
}
