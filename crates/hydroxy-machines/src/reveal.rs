//! Reveal: content that shows itself once it scrolls into view. A one-way
//! latch from hidden to shown. The adapter watches the viewport (an
//! `IntersectionObserver`) and reports back; the machine never sees geometry.
//!
//! It reveals at once when the user prefers reduced motion or the browser has
//! no observer. As a backstop it also reveals after a delay, so content can
//! never stay hidden if the observer never fires. The delay is a
//! [`RevealCommand::Wait`] the adapter runs; the machine reads no clock.

use hydroxy_statechart::{Machine, Transition};

/// Milliseconds before the backstop reveals content the observer never reported.
pub const BACKSTOP_MS: u32 = 1200;

/// Props for [`RevealMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RevealInput {
    /// Start shown, with no observing.
    pub default_shown: bool,
    /// The user prefers reduced motion: reveal at once.
    pub reduced_motion: bool,
    /// The browser can observe the viewport. Without it, reveal at once.
    pub can_observe: bool,
}

/// Why content was revealed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealReason {
    /// No animation wanted or possible.
    Immediate,
    /// It scrolled into view.
    Intersection,
    /// The backstop timer fired.
    Backstop,
    /// Asked for in code.
    Manual,
}

/// What [`RevealMachine`] remembers between events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RevealState {
    /// Content is shown. Never goes back to hidden.
    pub shown: bool,
    /// The adapter is observing the viewport.
    pub observing: bool,
    /// The backstop wait in flight.
    pub pending: Option<u32>,
    /// Last wait id handed out.
    pub seq: u32,
}

/// Inputs to [`RevealMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealEvent {
    /// The element is mounted: start watching.
    Start,
    /// The element entered the viewport.
    Intersect,
    /// Show it now.
    Reveal,
    /// The element is going away: stop watching.
    Stop,
    /// The wait with this id is over.
    Elapsed(u32),
}

/// Side effects from [`RevealMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealCommand {
    /// Begin an `IntersectionObserver` on the element.
    Observe,
    /// Disconnect it.
    Unobserve,
    /// Send [`RevealEvent::Elapsed`] with `id` after `ms` milliseconds.
    Wait {
        /// Echo this back in `Elapsed`.
        id: u32,
        /// How long to wait.
        ms: u32,
    },
    /// The content is now shown.
    Revealed(RevealReason),
}

/// Hidden-to-shown latch for one element.
#[derive(Debug, Clone, Copy)]
pub struct RevealMachine;

type Step = Transition<RevealState, RevealCommand>;

fn show(state: &RevealState, reason: RevealReason) -> Step {
    if state.shown {
        return Transition::stay();
    }
    let next = RevealState { shown: true, observing: false, pending: None, seq: state.seq };
    let mut commands = vec![RevealCommand::Revealed(reason)];
    if state.observing {
        commands.push(RevealCommand::Unobserve);
    }
    Transition::with(next, commands)
}

impl Machine for RevealMachine {
    type Input = RevealInput;
    type State = RevealState;
    type Event = RevealEvent;
    type Command = RevealCommand;

    fn initial(input: &RevealInput) -> RevealState {
        RevealState { shown: input.default_shown, ..RevealState::default() }
    }

    fn transition(state: &RevealState, event: RevealEvent, input: &RevealInput) -> Step {
        match event {
            RevealEvent::Start if !state.shown && !state.observing => {
                if input.reduced_motion || !input.can_observe {
                    return show(state, RevealReason::Immediate);
                }
                let id = state.seq.wrapping_add(1);
                Transition::with(
                    RevealState { observing: true, pending: Some(id), seq: id, ..*state },
                    vec![RevealCommand::Observe, RevealCommand::Wait { id, ms: BACKSTOP_MS }],
                )
            }
            RevealEvent::Intersect => show(state, RevealReason::Intersection),
            RevealEvent::Reveal => show(state, RevealReason::Manual),
            RevealEvent::Elapsed(id) if state.pending == Some(id) => show(state, RevealReason::Backstop),
            RevealEvent::Stop if state.observing => Transition::with(
                RevealState { observing: false, pending: None, ..*state },
                vec![RevealCommand::Unobserve],
            ),
            _ => Transition::stay(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OBSERVING: RevealInput = RevealInput { default_shown: false, reduced_motion: false, can_observe: true };

    #[allow(clippy::trivially_copy_pass_by_ref, reason = "keeps call sites uniform with the other machines' tests")]
    fn run(
        state: &RevealState,
        event: RevealEvent,
        input: &RevealInput,
    ) -> hydroxy_statechart::Step<RevealState, RevealCommand> {
        RevealMachine::transition(state, event, input).resolve(state)
    }

    #[test]
    fn start_observes_and_arms_the_backstop() {
        let s = RevealMachine::initial(&OBSERVING);
        let step = run(&s, RevealEvent::Start, &OBSERVING);
        assert_eq!(step.commands, [RevealCommand::Observe, RevealCommand::Wait { id: 1, ms: BACKSTOP_MS }]);
        assert!(!step.state.shown);
        assert!(run(&step.state, RevealEvent::Start, &OBSERVING).commands.is_empty(), "start is idempotent");
    }

    #[test]
    fn intersection_reveals_once_and_stops_watching() {
        let s = run(&RevealMachine::initial(&OBSERVING), RevealEvent::Start, &OBSERVING).state;
        let shown = run(&s, RevealEvent::Intersect, &OBSERVING);
        assert!(shown.state.shown);
        assert_eq!(shown.commands, [RevealCommand::Revealed(RevealReason::Intersection), RevealCommand::Unobserve]);
        assert!(run(&shown.state, RevealEvent::Intersect, &OBSERVING).commands.is_empty());
        assert!(run(&shown.state, RevealEvent::Elapsed(1), &OBSERVING).commands.is_empty(), "stale backstop");
    }

    #[test]
    fn the_backstop_reveals_what_the_observer_missed() {
        let s = run(&RevealMachine::initial(&OBSERVING), RevealEvent::Start, &OBSERVING).state;
        assert!(run(&s, RevealEvent::Elapsed(9), &OBSERVING).commands.is_empty());
        let step = run(&s, RevealEvent::Elapsed(1), &OBSERVING);
        assert_eq!(step.commands[0], RevealCommand::Revealed(RevealReason::Backstop));
    }

    #[test]
    fn reduced_motion_or_no_observer_reveals_immediately() {
        for input in
            [RevealInput { reduced_motion: true, ..OBSERVING }, RevealInput { can_observe: false, ..OBSERVING }]
        {
            let step = run(&RevealMachine::initial(&input), RevealEvent::Start, &input);
            assert_eq!(step.commands, [RevealCommand::Revealed(RevealReason::Immediate)]);
        }
    }

    #[test]
    fn stop_while_hidden_cancels_and_can_restart() {
        let s = run(&RevealMachine::initial(&OBSERVING), RevealEvent::Start, &OBSERVING).state;
        let stopped = run(&s, RevealEvent::Stop, &OBSERVING);
        assert_eq!(stopped.commands, [RevealCommand::Unobserve]);
        assert!(run(&stopped.state, RevealEvent::Elapsed(1), &OBSERVING).commands.is_empty());
        assert_eq!(
            run(&stopped.state, RevealEvent::Start, &OBSERVING).commands[1],
            RevealCommand::Wait { id: 2, ms: BACKSTOP_MS }
        );
    }

    #[test]
    fn default_shown_never_observes() {
        let input = RevealInput { default_shown: true, ..OBSERVING };
        let s = RevealMachine::initial(&input);
        assert!(s.shown);
        assert!(run(&s, RevealEvent::Start, &input).commands.is_empty());
    }
}
