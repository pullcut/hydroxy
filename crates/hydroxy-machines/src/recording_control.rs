//! Recording control: start, pause, resume and stop for a microphone recording.
//!
//! The browser does the recording. The machine decides what each press means and
//! counts only the time spent recording: time arrives as `Tick` events carrying a
//! monotonic clock reading in milliseconds, and paused time is not counted.

use hydroxy_statechart::{Machine, Transition};

/// Where the recorder is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RecordingPhase {
    /// Ready to record.
    #[default]
    Idle,
    /// Waiting for the microphone permission prompt.
    Requesting,
    /// Recording.
    Recording,
    /// Recording is held.
    Paused,
    /// Finishing: waiting for the recorder to hand over the audio.
    Stopping,
    /// A recording is ready.
    Ready,
    /// Something went wrong. See [`RecordingError`].
    Error,
}

impl RecordingPhase {
    /// The `data-state` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RecordingPhase::Idle => "idle",
            RecordingPhase::Requesting => "requesting",
            RecordingPhase::Recording => "recording",
            RecordingPhase::Paused => "paused",
            RecordingPhase::Stopping => "stopping",
            RecordingPhase::Ready => "ready",
            RecordingPhase::Error => "error",
        }
    }
}

/// Why recording failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingError {
    /// The microphone was refused.
    Denied,
    /// There is no microphone.
    NoDevice,
    /// This browser cannot record.
    Unsupported,
    /// The recorder broke off.
    Failed,
}

impl RecordingError {
    /// A sentence for the status line.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            RecordingError::Denied => "Microphone access was blocked. Allow it in your browser settings.",
            RecordingError::NoDevice => "No microphone was found.",
            RecordingError::Unsupported => "Recording is not available here.",
            RecordingError::Failed => "The recording stopped unexpectedly.",
        }
    }
}

/// Props for [`RecordingControlMachine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecordingControlInput {
    /// Stop on its own after this many milliseconds of recording. `None` for no limit.
    pub max_ms: Option<u64>,
    /// Ignore presses.
    pub disabled: bool,
    /// The recorder cannot pause, so the pause button stays hidden.
    pub no_pause: bool,
}

/// What [`RecordingControlMachine`] remembers between events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecordingControlState {
    /// The recorder phase.
    pub phase: RecordingPhase,
    /// Milliseconds recorded, not counting pauses.
    pub elapsed_ms: u64,
    /// The last clock reading, while the clock is running.
    pub last_tick: Option<u64>,
    /// Set in [`RecordingPhase::Error`].
    pub error: Option<RecordingError>,
}

/// Inputs to [`RecordingControlMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingControlEvent {
    /// Start (or record again).
    Start,
    /// Pause while recording.
    Pause,
    /// Continue after a pause.
    Resume,
    /// Finish the recording.
    Stop,
    /// Throw away a finished recording and start over.
    Discard,
    /// The microphone was granted and recording began.
    Granted,
    /// The recorder delivered its audio.
    Stopped,
    /// Recording failed.
    Failed(RecordingError),
    /// A monotonic clock reading in milliseconds.
    Tick(u64),
}

/// Side effects from [`RecordingControlMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingControlCommand {
    /// Ask for the microphone and start recording.
    Start,
    /// Pause the recorder.
    Pause,
    /// Resume the recorder.
    Resume,
    /// Stop the recorder and deliver the audio.
    Stop,
    /// Release the previous recording's audio.
    Release,
}

/// Idle, requesting, recording, paused, stopping, ready or error.
#[derive(Debug, Clone, Copy)]
pub struct RecordingControlMachine;

type Step = Transition<RecordingControlState, RecordingControlCommand>;

impl RecordingControlState {
    /// Start is available.
    #[must_use]
    pub fn can_start(&self) -> bool {
        matches!(self.phase, RecordingPhase::Idle | RecordingPhase::Ready | RecordingPhase::Error)
    }

    /// Stop is available.
    #[must_use]
    pub fn can_stop(&self) -> bool {
        matches!(self.phase, RecordingPhase::Recording | RecordingPhase::Paused)
    }

    /// Pause or resume is available.
    #[must_use]
    pub fn can_pause(&self, input: &RecordingControlInput) -> bool {
        !input.no_pause && matches!(self.phase, RecordingPhase::Recording | RecordingPhase::Paused)
    }
}

impl Machine for RecordingControlMachine {
    type Input = RecordingControlInput;
    type State = RecordingControlState;
    type Event = RecordingControlEvent;
    type Command = RecordingControlCommand;

    fn initial(_input: &RecordingControlInput) -> RecordingControlState {
        RecordingControlState::default()
    }

    fn transition(state: &RecordingControlState, event: RecordingControlEvent, input: &RecordingControlInput) -> Step {
        use RecordingControlCommand as C;
        use RecordingControlEvent as E;
        use RecordingPhase as P;
        let mut next = *state;
        let mut commands = Vec::new();
        match (event, state.phase) {
            (E::Start | E::Pause | E::Resume | E::Stop | E::Discard, _) if input.disabled => {}
            (E::Start, P::Idle | P::Ready | P::Error) => {
                if state.phase == P::Ready {
                    commands.push(C::Release);
                }
                next = RecordingControlState { phase: P::Requesting, ..RecordingControlState::default() };
                commands.push(C::Start);
            }
            (E::Discard, P::Ready) => {
                next = RecordingControlState::default();
                commands.push(C::Release);
            }
            (E::Pause, P::Recording) if !input.no_pause => {
                next.phase = P::Paused;
                next.last_tick = None;
                commands.push(C::Pause);
            }
            (E::Resume, P::Paused) => {
                next.phase = P::Recording;
                commands.push(C::Resume);
            }
            (E::Stop, P::Recording | P::Paused) => {
                next.phase = P::Stopping;
                next.last_tick = None;
                commands.push(C::Stop);
            }
            (E::Granted, P::Requesting) => next.phase = P::Recording,
            (E::Stopped, P::Recording | P::Paused | P::Stopping | P::Requesting) => {
                next.phase = P::Ready;
                next.last_tick = None;
            }
            (E::Failed(error), P::Requesting | P::Recording | P::Paused | P::Stopping) => {
                next =
                    RecordingControlState { phase: P::Error, error: Some(error), ..RecordingControlState::default() };
            }
            (E::Tick(now), P::Recording) => {
                if let Some(last) = state.last_tick {
                    next.elapsed_ms = state.elapsed_ms + now.saturating_sub(last);
                }
                next.last_tick = Some(now);
                if input.max_ms.is_some_and(|max| next.elapsed_ms >= max) {
                    next.phase = P::Stopping;
                    next.last_tick = None;
                    commands.push(C::Stop);
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
    use RecordingControlCommand as C;
    use RecordingControlEvent as E;
    use RecordingPhase as P;

    fn run(
        s: RecordingControlState,
        e: E,
        input: &RecordingControlInput,
    ) -> hydroxy_statechart::Step<RecordingControlState, RecordingControlCommand> {
        hydroxy_statechart::step::<RecordingControlMachine>(&s, e, input)
    }

    #[test]
    fn full_cycle() {
        let input = RecordingControlInput::default();
        let t = run(RecordingControlState::default(), E::Start, &input);
        assert_eq!((t.state.phase, t.commands), (P::Requesting, vec![C::Start]));
        let t = run(t.state, E::Granted, &input);
        let t = run(t.state, E::Stop, &input);
        assert_eq!((t.state.phase, t.commands), (P::Stopping, vec![C::Stop]));
        let t = run(t.state, E::Stopped, &input);
        assert_eq!(t.state.phase, P::Ready);
        let t = run(t.state, E::Start, &input);
        assert_eq!(t.commands, [C::Release, C::Start]);
    }

    #[test]
    fn paused_time_is_not_counted() {
        let input = RecordingControlInput::default();
        let mut s = RecordingControlState { phase: P::Recording, ..RecordingControlState::default() };
        s = run(s, E::Tick(1000), &input).state;
        s = run(s, E::Tick(2000), &input).state;
        assert_eq!(s.elapsed_ms, 1000);
        s = run(s, E::Pause, &input).state;
        s = run(s, E::Tick(9000), &input).state;
        assert_eq!(s.elapsed_ms, 1000);
        s = run(s, E::Resume, &input).state;
        s = run(s, E::Tick(10_000), &input).state;
        s = run(s, E::Tick(10_500), &input).state;
        assert_eq!(s.elapsed_ms, 1500);
    }

    #[test]
    fn limit_stops_by_itself() {
        let input = RecordingControlInput { max_ms: Some(1000), ..RecordingControlInput::default() };
        let mut s = RecordingControlState { phase: P::Recording, ..RecordingControlState::default() };
        s = run(s, E::Tick(0), &input).state;
        assert!(run(s, E::Tick(999), &input).commands.is_empty());
        let t = run(s, E::Tick(1000), &input);
        assert_eq!((t.state.phase, t.commands), (P::Stopping, vec![C::Stop]));
    }

    #[test]
    fn errors_can_retry_and_disabled_ignores() {
        let input = RecordingControlInput::default();
        let t = run(
            RecordingControlState { phase: P::Requesting, ..RecordingControlState::default() },
            E::Failed(RecordingError::Denied),
            &input,
        );
        assert_eq!((t.state.phase, t.state.error), (P::Error, Some(RecordingError::Denied)));
        assert_eq!(run(t.state, E::Start, &input).state.phase, P::Requesting);
        let off = RecordingControlInput { disabled: true, ..input };
        assert!(run(RecordingControlState::default(), E::Start, &off).commands.is_empty());
    }

    #[test]
    fn pause_hidden_when_unsupported() {
        let input = RecordingControlInput { no_pause: true, ..RecordingControlInput::default() };
        let s = RecordingControlState { phase: P::Recording, ..RecordingControlState::default() };
        assert!(!s.can_pause(&input));
        assert!(run(s, E::Pause, &input).commands.is_empty());
    }
}
