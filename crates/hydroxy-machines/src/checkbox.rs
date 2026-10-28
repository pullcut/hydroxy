//! Checkbox: checked, unchecked or indeterminate. A user toggle only ever
//! proposes checked or unchecked; only the owner can set indeterminate.

use hydroxy_statechart::{Controllable, Machine, Transition};

pub use crate::ChangeReason;

/// The three checkbox states.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CheckedState {
    /// Off.
    #[default]
    Unchecked,
    /// On.
    Checked,
    /// Mixed, e.g. a "select all" whose children disagree.
    Indeterminate,
}

impl From<bool> for CheckedState {
    fn from(checked: bool) -> Self {
        if checked { Self::Checked } else { Self::Unchecked }
    }
}

/// Props for [`CheckboxMachine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheckboxInput {
    /// Controlled state. `Some` at start fixes controlled mode.
    pub checked: Option<CheckedState>,
    /// Uncontrolled initial state, and what `Reset` restores.
    pub default_checked: CheckedState,
    /// Ignore toggles.
    pub disabled: bool,
}

/// What [`CheckboxMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheckboxState {
    /// The current state. Controlled when the owner holds it: the machine then only
    /// requests changes.
    pub checked: Controllable<CheckedState>,
}

/// Inputs to [`CheckboxMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxEvent {
    /// Click or Space.
    Toggle(ChangeReason),
    /// Back to `default_checked` (uncontrolled only), e.g. on form reset.
    Reset,
}

/// Side effects from [`CheckboxMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxCommand {
    /// Tell the owner the state changed (or should, when controlled).
    CheckedChange {
        /// The proposed state. Never indeterminate.
        checked: bool,
        /// What caused the change.
        reason: ChangeReason,
    },
}

/// Tri-state checkbox.
#[derive(Debug, Clone, Copy)]
pub struct CheckboxMachine;

impl Machine for CheckboxMachine {
    type Input = CheckboxInput;
    type State = CheckboxState;
    type Event = CheckboxEvent;
    type Command = CheckboxCommand;

    fn initial(input: &CheckboxInput) -> CheckboxState {
        CheckboxState {
            checked: Controllable::new(input.checked.unwrap_or(input.default_checked), input.checked.is_some()),
        }
    }

    fn transition(
        state: &CheckboxState,
        event: CheckboxEvent,
        input: &CheckboxInput,
    ) -> Transition<CheckboxState, CheckboxCommand> {
        let mut next = state.clone();
        match event {
            CheckboxEvent::Toggle(_) if input.disabled => Transition::to(next),
            CheckboxEvent::Toggle(reason) => {
                // Indeterminate becomes checked, like a native mixed checkbox.
                let checked = state.checked.value != CheckedState::Checked;
                next.checked.set(checked.into());
                Transition::with(next, vec![CheckboxCommand::CheckedChange { checked, reason }])
            }
            CheckboxEvent::Reset => {
                next.checked.set(input.default_checked);
                Transition::to(next)
            }
        }
    }

    fn sync(state: &CheckboxState, input: &CheckboxInput) -> Option<CheckboxState> {
        let mut next = state.clone();
        next.checked.sync(input.checked);
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn toggle(
        state: &CheckboxState,
        input: &CheckboxInput,
    ) -> hydroxy_statechart::Step<CheckboxState, CheckboxCommand> {
        advance::<CheckboxMachine>(state, CheckboxEvent::Toggle(ChangeReason::Pointer), input)
    }

    fn change(checked: bool) -> Vec<CheckboxCommand> {
        vec![CheckboxCommand::CheckedChange { checked, reason: ChangeReason::Pointer }]
    }

    #[test]
    fn uncontrolled_toggle_commits_and_requests() {
        let input = CheckboxInput::default();
        let step = toggle(&CheckboxMachine::initial(&input), &input);
        assert_eq!(step.state.checked.value, CheckedState::Checked);
        assert_eq!(step.commands, change(true));
        assert_eq!(toggle(&step.state, &input).commands, change(false));
    }

    #[test]
    fn indeterminate_toggles_to_checked() {
        let input = CheckboxInput { default_checked: CheckedState::Indeterminate, ..Default::default() };
        let step = toggle(&CheckboxMachine::initial(&input), &input);
        assert_eq!(step.state.checked.value, CheckedState::Checked);
        assert_eq!(step.commands, change(true));
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = CheckboxInput { checked: Some(CheckedState::Indeterminate), ..Default::default() };
        let step = toggle(&CheckboxMachine::initial(&input), &input);
        assert_eq!(step.state.checked.value, CheckedState::Indeterminate);
        assert_eq!(step.commands, change(true));
        input.checked = Some(CheckedState::Checked);
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<CheckboxMachine>(&step.state, &input).checked.value,
            CheckedState::Checked
        );
    }

    #[test]
    fn disabled_ignores_toggle() {
        let input = CheckboxInput { disabled: true, ..Default::default() };
        let step = toggle(&CheckboxMachine::initial(&input), &input);
        assert_eq!(step.state.checked.value, CheckedState::Unchecked);
        assert!(step.commands.is_empty());
    }

    #[test]
    fn reset_restores_default_when_uncontrolled() {
        let input = CheckboxInput { default_checked: CheckedState::Indeterminate, ..Default::default() };
        let checked = CheckboxState { checked: Controllable::new(CheckedState::Checked, false) };
        let step = advance::<CheckboxMachine>(&checked, CheckboxEvent::Reset, &input);
        assert_eq!(step.state.checked.value, CheckedState::Indeterminate);
        assert!(step.commands.is_empty());
    }
}
