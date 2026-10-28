//! Voice input: a record button's states. The browser does the recording; the
//! machine decides what the button means and when to stop.
//!
//! Time arrives as `Tick` events carrying the elapsed milliseconds. The machine
//! never reads a clock.

use hydroxy_statechart::{Machine, Transition};

/// Where the recorder is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VoiceInputPhase {
    /// Ready to record.
    #[default]
    Idle,
    /// Waiting for the microphone permission prompt.
    Requesting,
    /// Recording.
    Recording,
    /// The user (or the browser) refused the microphone.
    Denied,
    /// Recording is not possible here, or it failed.
    Failed,
}

impl VoiceInputPhase {
    /// The `data-state` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            VoiceInputPhase::Idle => "idle",
            VoiceInputPhase::Requesting => "requesting",
            VoiceInputPhase::Recording => "recording",
            VoiceInputPhase::Denied => "denied",
            VoiceInputPhase::Failed => "failed",
        }
    }
}

/// Props for [`VoiceInputMachine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VoiceInputInput {
    /// Stop on its own after this many milliseconds. `None` for no limit.
    pub max_ms: Option<u64>,
    /// Ignore presses.
    pub disabled: bool,
}

/// What [`VoiceInputMachine`] remembers between events.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct VoiceInputState {
    /// The recorder phase.
    pub phase: VoiceInputPhase,
    /// Input level from 0 to 1, for the meter.
    pub level: f32,
    /// Milliseconds recorded so far.
    pub elapsed_ms: u64,
}

/// Inputs to [`VoiceInputMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceInputEvent {
    /// The button was pressed: start when ready, stop when recording.
    Press,
    /// The microphone was granted and recording began.
    Granted,
    /// The microphone was refused.
    Denied,
    /// Recording could not start or broke off.
    Failed,
    /// The recorder finished and handed over its audio.
    Stopped,
    /// The input level changed (0 to 1).
    Level(f32),
    /// Elapsed recording time in milliseconds.
    Tick(u64),
}

/// Side effects from [`VoiceInputMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceInputCommand {
    /// Ask for the microphone and start recording.
    Start,
    /// Stop recording and deliver the audio.
    Stop,
}

/// Idle, requesting, recording, denied or failed.
#[derive(Debug, Clone, Copy)]
pub struct VoiceInputMachine;

type Step = Transition<VoiceInputState, VoiceInputCommand>;

fn phase(phase: VoiceInputPhase) -> VoiceInputState {
    VoiceInputState { phase, ..VoiceInputState::default() }
}

impl Machine for VoiceInputMachine {
    type Input = VoiceInputInput;
    type State = VoiceInputState;
    type Event = VoiceInputEvent;
    type Command = VoiceInputCommand;

    fn initial(_input: &VoiceInputInput) -> VoiceInputState {
        VoiceInputState::default()
    }

    fn transition(state: &VoiceInputState, event: VoiceInputEvent, input: &VoiceInputInput) -> Step {
        use VoiceInputPhase as P;
        let mut next = *state;
        let mut commands = Vec::new();
        match (event, state.phase) {
            (VoiceInputEvent::Press, _) if input.disabled => {}
            (VoiceInputEvent::Press, P::Idle | P::Denied | P::Failed) => {
                next = phase(P::Requesting);
                commands.push(VoiceInputCommand::Start);
            }
            (VoiceInputEvent::Press, P::Recording) => commands.push(VoiceInputCommand::Stop),
            (VoiceInputEvent::Granted, P::Requesting) => next.phase = P::Recording,
            (VoiceInputEvent::Denied, P::Requesting | P::Recording) => next = phase(P::Denied),
            (VoiceInputEvent::Failed, P::Requesting | P::Recording) => next = phase(P::Failed),
            (VoiceInputEvent::Stopped, P::Recording | P::Requesting) => next = phase(P::Idle),
            (VoiceInputEvent::Level(level), P::Recording) => next.level = level.clamp(0.0, 1.0),
            (VoiceInputEvent::Tick(ms), P::Recording) => {
                next.elapsed_ms = ms;
                if input.max_ms.is_some_and(|max| ms >= max) {
                    commands.push(VoiceInputCommand::Stop);
                }
            }
            _ => {}
        }
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use VoiceInputPhase as P;

    fn step(s: VoiceInputState, e: VoiceInputEvent, input: &VoiceInputInput) -> hydroxy_statechart::Step<VoiceInputState, VoiceInputCommand> {
        VoiceInputMachine::transition(&s, e, input).resolve(&s)
    }

    #[test]
    fn record_and_stop() {
        let input = VoiceInputInput::default();
        let s = VoiceInputMachine::initial(&input);
        let t = step(s, VoiceInputEvent::Press, &input);
        assert_eq!((t.state.phase, t.commands), (P::Requesting, vec![VoiceInputCommand::Start]));
        let t = step(t.state, VoiceInputEvent::Granted, &input);
        assert_eq!(t.state.phase, P::Recording);
        let t = step(t.state, VoiceInputEvent::Level(2.0), &input);
        assert!((t.state.level - 1.0).abs() < f32::EPSILON);
        let t = step(t.state, VoiceInputEvent::Press, &input);
        assert_eq!(t.commands, [VoiceInputCommand::Stop]);
        let t = step(t.state, VoiceInputEvent::Stopped, &input);
        assert_eq!(t.state, VoiceInputState::default());
    }

    #[test]
    fn denied_can_retry() {
        let input = VoiceInputInput::default();
        let s = phase(P::Requesting);
        let t = step(s, VoiceInputEvent::Denied, &input);
        assert_eq!(t.state.phase, P::Denied);
        assert_eq!(step(t.state, VoiceInputEvent::Press, &input).state.phase, P::Requesting);
    }

    #[test]
    fn limit_stops_and_disabled_ignores() {
        let input = VoiceInputInput { max_ms: Some(1000), disabled: false };
        let s = phase(P::Recording);
        assert!(step(s, VoiceInputEvent::Tick(999), &input).commands.is_empty());
        assert_eq!(step(s, VoiceInputEvent::Tick(1000), &input).commands, [VoiceInputCommand::Stop]);
        let off = VoiceInputInput { disabled: true, ..input };
        assert!(step(VoiceInputState::default(), VoiceInputEvent::Press, &off).commands.is_empty());
    }
}
