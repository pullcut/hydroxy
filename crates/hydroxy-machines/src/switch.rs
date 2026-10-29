//! Switch owns one on/off value. Uncontrolled presses commit and report;
//! controlled presses only request, and the owner's value arrives via `sync`.

use hydroxy_statechart::{Controllable, Machine, Transition};

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
    /// Whether the switch is on. Controlled when the owner holds it: the machine then
    /// only requests changes.
    pub checked: Controllable<bool>,
}

/// Inputs to [`SwitchMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchEvent {
    /// Flip the switch.
    Toggle(ChangeReason),
}

/// Side effects from [`SwitchMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        SwitchState {
            checked: Controllable::new(input.checked.unwrap_or(input.default_checked), input.checked.is_some()),
        }
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
        let checked = !state.checked.value;
        let mut next = state.clone();
        next.checked.set(checked);
        Transition::update(state, next, vec![SwitchCommand::CheckedChange { checked, reason }])
    }

    fn sync(state: &SwitchState, input: &SwitchInput) -> Option<SwitchState> {
        let mut next = state.clone();
        next.checked.sync(input.checked);
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn toggle(state: &SwitchState, input: &SwitchInput) -> hydroxy_statechart::Step<SwitchState, SwitchCommand> {
        step::<SwitchMachine>(state, SwitchEvent::Toggle(ChangeReason::Pointer), input)
    }

    #[test]
    fn uncontrolled_commits_and_reports() {
        let input = SwitchInput::default();
        let step = toggle(&SwitchMachine::initial(&input), &input);
        assert!(step.state.checked.value);
        assert_eq!(step.commands, vec![SwitchCommand::CheckedChange { checked: true, reason: ChangeReason::Pointer }]);
        assert!(!toggle(&step.state, &input).state.checked.value);
    }

    #[test]
    fn controlled_false_is_controlled_and_waits_for_sync() {
        let mut input = SwitchInput { checked: Some(false), ..SwitchInput::default() };
        let state = SwitchMachine::initial(&input);
        assert!(state.checked.controlled);

        let step = toggle(&state, &input);
        assert!(!step.state.checked.value);
        assert_eq!(step.commands, vec![SwitchCommand::CheckedChange { checked: true, reason: ChangeReason::Pointer }]);

        input.checked = Some(true);
        assert!(hydroxy_statechart::sync_or_keep::<SwitchMachine>(&step.state, &input).checked.value);
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
        assert_eq!(hydroxy_statechart::sync_or_keep::<SwitchMachine>(&state, &SwitchInput::default()), state);
    }
}
