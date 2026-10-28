//! Voice session: the states of a live voice conversation.
//!
//! The audio transport belongs to the owner. In controlled mode the owner holds
//! the phase and the machine only asks for changes; the owner reports what the
//! transport is doing through `phase`. Uncontrolled, the machine tracks the
//! phase itself from events sent to it.

use hydroxy_statechart::{Machine, Transition};

/// Where a voice conversation is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VoicePhase {
    /// No call.
    #[default]
    Idle,
    /// Setting up the connection.
    Connecting,
    /// Connected; the user's turn to talk.
    Listening,
    /// Connected; the assistant is talking.
    Speaking,
    /// The connection failed or dropped.
    Error,
}

impl VoicePhase {
    /// The `data-state` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            VoicePhase::Idle => "idle",
            VoicePhase::Connecting => "connecting",
            VoicePhase::Listening => "listening",
            VoicePhase::Speaking => "speaking",
            VoicePhase::Error => "error",
        }
    }

    /// A call is up: listening or speaking.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, VoicePhase::Listening | VoicePhase::Speaking)
    }
}

/// Props for [`VoiceSessionMachine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VoiceSessionInput {
    /// Controlled phase. `Some` at start fixes controlled mode.
    pub phase: Option<VoicePhase>,
    /// Uncontrolled initial phase.
    pub default_phase: VoicePhase,
    /// Controlled mute. `Some` at start fixes controlled mode.
    pub muted: Option<bool>,
    /// Uncontrolled initial mute.
    pub default_muted: bool,
}

/// What [`VoiceSessionMachine`] remembers between events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VoiceSessionState {
    /// The conversation phase.
    pub phase: VoicePhase,
    /// The microphone is muted.
    pub muted: bool,
    /// Fixed at start: the owner holds the phase.
    pub phase_controlled: bool,
    /// Fixed at start: the owner holds the mute flag.
    pub muted_controlled: bool,
}

/// Inputs to [`VoiceSessionMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceSessionEvent {
    /// The user asked to start (or retry after an error).
    Start,
    /// The user ended the call, or cancelled while connecting.
    End,
    /// The user flipped the mute button.
    ToggleMute,
    /// The transport connected.
    Connected,
    /// The assistant stopped talking.
    Listening,
    /// The assistant started talking.
    Speaking,
    /// The transport failed.
    Failed,
}

/// Side effects from [`VoiceSessionMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceSessionCommand {
    /// Tell the owner to open the transport.
    Start,
    /// Tell the owner to close the transport.
    End,
    /// Tell the owner the phase changed (or should, when controlled).
    PhaseChange(VoicePhase),
    /// Tell the owner the mute flag changed (or should, when controlled).
    MuteChange(bool),
}

/// Idle, connecting, listening, speaking or error, plus mute.
#[derive(Debug, Clone, Copy)]
pub struct VoiceSessionMachine;

type Step = Transition<VoiceSessionState, VoiceSessionCommand>;

fn go(state: VoiceSessionState, phase: VoicePhase, mut commands: Vec<VoiceSessionCommand>) -> Step {
    if phase == state.phase {
        return Transition::to(state);
    }
    commands.push(VoiceSessionCommand::PhaseChange(phase));
    let mut next = state;
    if !state.phase_controlled {
        next.phase = phase;
    }
    Transition::with(next, commands)
}

impl Machine for VoiceSessionMachine {
    type Input = VoiceSessionInput;
    type State = VoiceSessionState;
    type Event = VoiceSessionEvent;
    type Command = VoiceSessionCommand;

    fn initial(input: &VoiceSessionInput) -> VoiceSessionState {
        VoiceSessionState {
            phase: input.phase.unwrap_or(input.default_phase),
            muted: input.muted.unwrap_or(input.default_muted),
            phase_controlled: input.phase.is_some(),
            muted_controlled: input.muted.is_some(),
        }
    }

    fn transition(state: &VoiceSessionState, event: VoiceSessionEvent, _input: &VoiceSessionInput) -> Step {
        use VoicePhase as P;
        use VoiceSessionCommand as C;
        match (event, state.phase) {
            (VoiceSessionEvent::Start, P::Idle | P::Error) => go(*state, P::Connecting, vec![C::Start]),
            (VoiceSessionEvent::End, P::Connecting | P::Listening | P::Speaking | P::Error) => {
                go(*state, P::Idle, vec![C::End])
            }
            (VoiceSessionEvent::ToggleMute, p) if p.is_live() => {
                let muted = !state.muted;
                let mut next = *state;
                if !state.muted_controlled {
                    next.muted = muted;
                }
                Transition::with(next, vec![C::MuteChange(muted)])
            }
            (VoiceSessionEvent::Connected | VoiceSessionEvent::Listening, P::Connecting | P::Speaking) => {
                go(*state, P::Listening, vec![])
            }
            (VoiceSessionEvent::Speaking, P::Connecting | P::Listening) => go(*state, P::Speaking, vec![]),
            (VoiceSessionEvent::Failed, P::Connecting | P::Listening | P::Speaking) => go(*state, P::Error, vec![]),
            _ => Transition::stay(),
        }
    }

    fn sync(state: &VoiceSessionState, input: &VoiceSessionInput) -> Option<VoiceSessionState> {
        let mut next = *state;
        if let (true, Some(phase)) = (state.phase_controlled, input.phase) {
            next.phase = phase;
        }
        if let (true, Some(muted)) = (state.muted_controlled, input.muted) {
            next.muted = muted;
        }
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use VoicePhase as P;
    use hydroxy_statechart::step;

    fn run(
        input: VoiceSessionInput,
        state: VoiceSessionState,
        event: VoiceSessionEvent,
    ) -> hydroxy_statechart::Step<VoiceSessionState, VoiceSessionCommand> {
        step::<VoiceSessionMachine>(&state, event, &input)
    }

    #[test]
    fn uncontrolled_walks_a_call() {
        let input = VoiceSessionInput::default();
        let s = VoiceSessionMachine::initial(&input);
        let step = run(input, s, VoiceSessionEvent::Start);
        assert_eq!(step.state.phase, P::Connecting);
        assert_eq!(step.commands, [VoiceSessionCommand::Start, VoiceSessionCommand::PhaseChange(P::Connecting)]);
        let step = run(input, step.state, VoiceSessionEvent::Connected);
        assert_eq!(step.state.phase, P::Listening);
        let step = run(input, step.state, VoiceSessionEvent::Speaking);
        assert_eq!(step.state.phase, P::Speaking);
        let step = run(input, step.state, VoiceSessionEvent::End);
        assert_eq!(step.state.phase, P::Idle);
        assert_eq!(step.commands[0], VoiceSessionCommand::End);
    }

    #[test]
    fn controlled_phase_only_requests() {
        let input = VoiceSessionInput { phase: Some(P::Idle), ..Default::default() };
        let s = VoiceSessionMachine::initial(&input);
        let step = run(input, s, VoiceSessionEvent::Start);
        assert_eq!(step.state.phase, P::Idle);
        assert!(step.commands.contains(&VoiceSessionCommand::Start));
        let input = VoiceSessionInput { phase: Some(P::Listening), ..Default::default() };
        assert_eq!(hydroxy_statechart::sync_or_keep::<VoiceSessionMachine>(&step.state, &input).phase, P::Listening);
    }

    #[test]
    fn mute_only_while_live_and_failure_can_retry() {
        let input = VoiceSessionInput::default();
        let s = VoiceSessionMachine::initial(&input);
        assert!(run(input, s, VoiceSessionEvent::ToggleMute).commands.is_empty());
        let live = VoiceSessionState { phase: P::Listening, ..s };
        let step = run(input, live, VoiceSessionEvent::ToggleMute);
        assert!(step.state.muted);
        assert_eq!(step.commands, [VoiceSessionCommand::MuteChange(true)]);
        let failed = run(input, live, VoiceSessionEvent::Failed).state;
        assert_eq!(failed.phase, P::Error);
        assert_eq!(run(input, failed, VoiceSessionEvent::Start).state.phase, P::Connecting);
    }
}
