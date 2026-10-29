//! Wizard owns the current step of a multi-step flow and which steps are done.
//! Validation lives with the owner: the adapter asks it before sending `Next`,
//! so `Next` here always means "this step passed". In linear mode a step can
//! only be visited once every step before it is done.

use hydroxy_statechart::{Machine, Transition};

pub use crate::ChangeReason;

/// Props for [`WizardMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WizardInput {
    /// Number of steps.
    pub count: usize,
    /// Controlled step. `Some` at start fixes controlled mode.
    pub step: Option<usize>,
    /// Uncontrolled initial step.
    pub default_step: usize,
    /// Steps unlock in order. Off: any step can be visited at any time.
    pub linear: bool,
}

impl WizardInput {
    fn last(&self) -> usize {
        self.count.saturating_sub(1)
    }
}

/// What [`WizardMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WizardState {
    /// The step on screen.
    pub current: usize,
    /// Done flags, one per step.
    pub completed: Vec<bool>,
    /// Fixed at start: the owner holds the step and the machine only requests changes.
    pub controlled: bool,
}

impl WizardState {
    /// Whether step `index` is done.
    #[must_use]
    pub fn is_complete(&self, index: usize) -> bool {
        self.completed.get(index).copied().unwrap_or(false)
    }

    /// Whether step `index` can be visited now.
    #[must_use]
    pub fn can_visit(&self, index: usize, input: &WizardInput) -> bool {
        index < input.count && (!input.linear || index <= self.current || (0..index).all(|k| self.is_complete(k)))
    }
}

/// Inputs to [`WizardMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WizardEvent {
    /// The current step passed validation: mark it done and move on (or finish on the last step).
    Next(ChangeReason),
    /// Back one step.
    Back(ChangeReason),
    /// Jump to a step, if it can be visited.
    GoTo(usize, ChangeReason),
}

/// Side effects from [`WizardMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WizardCommand {
    /// Tell the owner the step changed (or should, when controlled).
    StepChange {
        /// The new (or proposed) step.
        step: usize,
        /// What caused the change.
        reason: ChangeReason,
    },
    /// Next was pressed on the last step, after it passed.
    Finish,
}

/// A multi-step flow.
#[derive(Debug, Clone, Copy)]
pub struct WizardMachine;

impl Machine for WizardMachine {
    type Input = WizardInput;
    type State = WizardState;
    type Event = WizardEvent;
    type Command = WizardCommand;

    fn initial(input: &WizardInput) -> WizardState {
        WizardState {
            current: input.step.unwrap_or(input.default_step).min(input.last()),
            completed: vec![false; input.count],
            controlled: input.step.is_some(),
        }
    }

    fn transition(
        state: &WizardState,
        event: WizardEvent,
        input: &WizardInput,
    ) -> Transition<WizardState, WizardCommand> {
        if input.count == 0 {
            return Transition::stay();
        }
        let mut next = state.clone();
        let (target, reason) = match event {
            WizardEvent::Next(reason) => {
                if let Some(done) = next.completed.get_mut(state.current) {
                    *done = true;
                }
                if state.current >= input.last() {
                    return Transition::update(state, next, vec![WizardCommand::Finish]);
                }
                (state.current + 1, reason)
            }
            WizardEvent::Back(reason) => (state.current.saturating_sub(1), reason),
            WizardEvent::GoTo(index, reason) => {
                if !state.can_visit(index, input) {
                    return Transition::update(state, next, Vec::new());
                }
                (index, reason)
            }
        };
        if target == state.current {
            return Transition::update(state, next, Vec::new());
        }
        if !state.controlled {
            next.current = target;
        }
        Transition::update(state, next, vec![WizardCommand::StepChange { step: target, reason }])
    }

    fn sync(state: &WizardState, input: &WizardInput) -> Option<WizardState> {
        let mut next = state.clone();
        next.completed.resize(input.count, false);
        let current = match (state.controlled, input.step) {
            (true, Some(step)) => step,
            _ => state.current,
        };
        next.current = current.min(input.last());
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: ChangeReason = ChangeReason::Pointer;

    fn linear() -> WizardInput {
        WizardInput { count: 3, linear: true, ..WizardInput::default() }
    }

    #[test]
    fn next_completes_and_advances_then_finishes() {
        let input = linear();
        let step =
            hydroxy_statechart::step::<WizardMachine>(&WizardMachine::initial(&input), WizardEvent::Next(P), &input);
        assert_eq!(step.state.current, 1);
        assert!(step.state.is_complete(0));
        assert_eq!(step.commands, vec![WizardCommand::StepChange { step: 1, reason: P }]);
        let step = hydroxy_statechart::step::<WizardMachine>(&step.state, WizardEvent::Next(P), &input);
        let step = hydroxy_statechart::step::<WizardMachine>(&step.state, WizardEvent::Next(P), &input);
        assert_eq!(step.state.current, 2);
        assert!(step.state.is_complete(2));
        assert_eq!(step.commands, vec![WizardCommand::Finish]);
    }

    #[test]
    fn back_clamps_at_first() {
        let input = linear();
        let state = WizardMachine::initial(&input);
        assert!(hydroxy_statechart::step::<WizardMachine>(&state, WizardEvent::Back(P), &input).commands.is_empty());
        let step = hydroxy_statechart::step::<WizardMachine>(&state, WizardEvent::Next(P), &input);
        assert_eq!(
            hydroxy_statechart::step::<WizardMachine>(&step.state, WizardEvent::Back(P), &input).state.current,
            0
        );
    }

    #[test]
    fn linear_blocks_jumping_past_unfinished_steps() {
        let input = linear();
        let state = WizardMachine::initial(&input);
        assert!(!state.can_visit(1, &input));
        assert!(hydroxy_statechart::step::<WizardMachine>(&state, WizardEvent::GoTo(2, P), &input).commands.is_empty());
        let step = hydroxy_statechart::step::<WizardMachine>(&state, WizardEvent::Next(P), &input);
        let step = hydroxy_statechart::step::<WizardMachine>(&step.state, WizardEvent::Back(P), &input);
        // Step 0 is done, so step 1 is open again; step 2 is not.
        assert!(step.state.can_visit(1, &input));
        assert!(!step.state.can_visit(2, &input));
        assert_eq!(
            hydroxy_statechart::step::<WizardMachine>(&step.state, WizardEvent::GoTo(1, P), &input).state.current,
            1
        );
    }

    #[test]
    fn free_mode_visits_anything() {
        let input = WizardInput { linear: false, ..linear() };
        let step =
            hydroxy_statechart::step::<WizardMachine>(&WizardMachine::initial(&input), WizardEvent::GoTo(2, P), &input);
        assert_eq!(step.state.current, 2);
        assert!(
            hydroxy_statechart::step::<WizardMachine>(&step.state, WizardEvent::GoTo(9, P), &input).commands.is_empty()
        );
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = WizardInput { step: Some(0), ..linear() };
        let step =
            hydroxy_statechart::step::<WizardMachine>(&WizardMachine::initial(&input), WizardEvent::Next(P), &input);
        assert_eq!(step.state.current, 0);
        assert_eq!(step.commands, vec![WizardCommand::StepChange { step: 1, reason: P }]);
        input.step = Some(1);
        assert_eq!(hydroxy_statechart::sync_or_keep::<WizardMachine>(&step.state, &input).current, 1);
    }

    #[test]
    fn sync_follows_step_count() {
        let input = linear();
        let state = WizardMachine::initial(&WizardInput { default_step: 2, ..input.clone() });
        let fewer = WizardInput { count: 2, ..input };
        let synced = hydroxy_statechart::sync_or_keep::<WizardMachine>(&state, &fewer);
        assert_eq!(synced.current, 1);
        assert_eq!(synced.completed.len(), 2);
    }
}
