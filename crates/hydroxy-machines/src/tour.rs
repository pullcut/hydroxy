//! Tour: onboarding coachmarks, one step at a time. The machine owns whether
//! the tour is running and which step shows; the adapter spotlights the step's
//! target, scrolls it into view, moves focus into the popover and gives focus
//! back when the tour ends.

use hydroxy_statechart::{Machine, Transition};

/// Props for [`TourMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TourInput {
    /// Number of steps.
    pub count: usize,
    /// Controlled running state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled: start running.
    pub default_open: bool,
}

/// What [`TourMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TourState {
    /// Whether the tour is showing.
    pub open: bool,
    /// The current step.
    pub index: usize,
    /// Fixed at start: the owner holds `open`.
    pub controlled: bool,
}

/// How a tour ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourEnd {
    /// "Done" on the last step.
    Completed,
    /// Skip, Escape, or closed by the owner, at this step.
    Skipped(usize),
}

/// Inputs to [`TourMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourEvent {
    /// Start from the first step.
    Start,
    /// Next step; on the last step, finish.
    Next,
    /// Previous step. No-op on the first.
    Prev,
    /// Jump to a step.
    Go(usize),
    /// End early (Skip button or Escape).
    Skip,
}

/// Side effects from [`TourMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourCommand {
    /// The tour opened or closed (or should, when controlled).
    OpenChange(bool),
    /// A new step is showing.
    StepChange(usize),
    /// The tour ended, and how.
    End(TourEnd),
}

/// Running state and step for one tour.
#[derive(Debug, Clone, Copy)]
pub struct TourMachine;

type Step = Transition<TourState, TourCommand>;

fn close(state: &TourState, how: TourEnd) -> Step {
    let mut next = state.clone();
    if !state.controlled {
        next.open = false;
    }
    Transition::with(next, vec![TourCommand::OpenChange(false), TourCommand::End(how)])
}

fn go(state: &TourState, index: usize) -> Step {
    if index == state.index {
        return Transition::stay();
    }
    let mut next = state.clone();
    next.index = index;
    Transition::with(next, vec![TourCommand::StepChange(index)])
}

impl TourState {
    /// Whether the current step is the last.
    #[must_use]
    pub fn is_last(&self, input: &TourInput) -> bool {
        self.index + 1 >= input.count
    }
}

impl Machine for TourMachine {
    type Input = TourInput;
    type State = TourState;
    type Event = TourEvent;
    type Command = TourCommand;

    fn initial(input: &TourInput) -> TourState {
        TourState { open: input.open.unwrap_or(input.default_open), index: 0, controlled: input.open.is_some() }
    }

    fn transition(state: &TourState, event: TourEvent, input: &TourInput) -> Step {
        if input.count == 0 {
            return Transition::stay();
        }
        match event {
            TourEvent::Start => {
                let mut next = state.clone();
                next.index = 0;
                if !state.controlled {
                    next.open = true;
                }
                Transition::with(next, vec![TourCommand::OpenChange(true), TourCommand::StepChange(0)])
            }
            _ if !state.open => Transition::stay(),
            TourEvent::Next if state.is_last(input) => close(state, TourEnd::Completed),
            TourEvent::Next => go(state, state.index + 1),
            TourEvent::Prev => go(state, state.index.saturating_sub(1)),
            TourEvent::Go(i) => go(state, i.min(input.count - 1)),
            TourEvent::Skip => close(state, TourEnd::Skipped(state.index)),
        }
    }

    fn sync(state: &TourState, input: &TourInput) -> Option<TourState> {
        let mut next = state.clone();
        if let (true, Some(open)) = (state.controlled, input.open) {
            if open && !state.open {
                next.index = 0;
            }
            next.open = open;
        }
        next.index = next.index.min(input.count.saturating_sub(1));
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn input() -> TourInput {
        TourInput { count: 3, ..Default::default() }
    }

    fn run(state: &TourState, event: TourEvent) -> hydroxy_statechart::Step<TourState, TourCommand> {
        step::<TourMachine>(state, event, &input())
    }

    #[test]
    fn start_opens_on_the_first_step() {
        let step = run(&TourState { index: 2, ..Default::default() }, TourEvent::Start);
        assert!(step.state.open);
        assert_eq!(step.state.index, 0);
        assert_eq!(step.commands, vec![TourCommand::OpenChange(true), TourCommand::StepChange(0)]);
    }

    #[test]
    fn next_walks_then_completes() {
        let mut s = run(&TourState::default(), TourEvent::Start).state;
        s = run(&s, TourEvent::Next).state;
        s = run(&s, TourEvent::Next).state;
        assert_eq!(s.index, 2);
        let end = run(&s, TourEvent::Next);
        assert!(!end.state.open);
        assert_eq!(end.commands, vec![TourCommand::OpenChange(false), TourCommand::End(TourEnd::Completed)]);
    }

    #[test]
    fn prev_stops_at_the_first_step() {
        let s = run(&TourState::default(), TourEvent::Start).state;
        assert!(run(&s, TourEvent::Prev).commands.is_empty());
    }

    #[test]
    fn skip_reports_where_it_stopped() {
        let s = TourState { open: true, index: 1, controlled: false };
        let step = run(&s, TourEvent::Skip);
        assert_eq!(step.commands[1], TourCommand::End(TourEnd::Skipped(1)));
    }

    #[test]
    fn closed_tour_ignores_navigation() {
        assert!(run(&TourState::default(), TourEvent::Next).commands.is_empty());
    }

    #[test]
    fn go_clamps_to_the_last_step() {
        let s = TourState { open: true, index: 0, controlled: false };
        assert_eq!(run(&s, TourEvent::Go(9)).state.index, 2);
    }

    #[test]
    fn controlled_reopen_restarts() {
        let mut input = TourInput { count: 3, open: Some(true), default_open: false };
        let s = TourState { open: true, index: 2, controlled: true };
        input.open = Some(false);
        let closed = hydroxy_statechart::sync_or_keep::<TourMachine>(&s, &input);
        input.open = Some(true);
        assert_eq!(hydroxy_statechart::sync_or_keep::<TourMachine>(&closed, &input).index, 0);
    }
}
