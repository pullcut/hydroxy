//! Media timeline: the scrubber state for an `<audio>` or `<video>` element.
//!
//! The media element owns the clock. It reports `Time`, `Duration`, `Buffered`
//! and `Playing`; the machine turns pointer and key input into `Seek`, `Play`
//! and `Pause` commands. While the user scrubs, reported times are ignored so
//! the thumb follows the pointer.

use oxide_statechart::{Machine, Transition};

/// Props for [`MediaTimelineMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaTimelineInput {
    /// Seconds moved by an arrow key. Shift uses a fifth of it.
    pub step: f64,
    /// Ignore all input.
    pub disabled: bool,
}

impl Default for MediaTimelineInput {
    fn default() -> Self {
        Self { step: 5.0, disabled: false }
    }
}

/// What [`MediaTimelineMachine`] remembers between events.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MediaTimelineState {
    /// Playback position in seconds.
    pub current: f64,
    /// Length in seconds. Zero until the media reports it.
    pub duration: f64,
    /// Seconds loaded from the start.
    pub buffered: f64,
    /// The media is playing.
    pub playing: bool,
    /// Time under the pointer, for the hover preview.
    pub hover: Option<f64>,
    /// The user is dragging the thumb.
    pub scrubbing: bool,
}

impl MediaTimelineState {
    /// Position as a fraction of the duration, 0 to 1.
    #[must_use]
    pub fn fraction(&self, seconds: f64) -> f64 {
        if self.duration > 0.0 { (seconds / self.duration).clamp(0.0, 1.0) } else { 0.0 }
    }
}

/// Inputs to [`MediaTimelineMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaTimelineEvent {
    /// The media's current time changed.
    Time(f64),
    /// The media's duration is known or changed.
    Duration(f64),
    /// The media's loaded range now reaches this second.
    Buffered(f64),
    /// The media started or stopped playing.
    Playing(bool),
    /// The pointer is over this time, or left the track.
    Hover(Option<f64>),
    /// Pointer down on the track at this time.
    ScrubStart(f64),
    /// Pointer moved to this time while held.
    ScrubMove(f64),
    /// Pointer released.
    ScrubEnd,
    /// Move by this many seconds (negative for back).
    Nudge(f64),
    /// Move by this fraction of the duration (negative for back).
    Page(f64),
    /// Home.
    ToStart,
    /// End.
    ToEnd,
    /// Space or K.
    TogglePlay,
}

/// Side effects from [`MediaTimelineMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaTimelineCommand {
    /// Set the media's `currentTime`.
    Seek(f64),
    /// Call `play()`.
    Play,
    /// Call `pause()`.
    Pause,
}

/// Position, buffered range, hover and scrubbing for a media element.
#[derive(Debug, Clone, Copy)]
pub struct MediaTimelineMachine;

type Step = Transition<MediaTimelineState, MediaTimelineCommand>;

fn seek(mut state: MediaTimelineState, to: f64) -> Step {
    let to = if to.is_finite() { to.clamp(0.0, state.duration.max(0.0)) } else { 0.0 };
    state.current = to;
    Transition::with(state, vec![MediaTimelineCommand::Seek(to)])
}

impl Machine for MediaTimelineMachine {
    type Input = MediaTimelineInput;
    type State = MediaTimelineState;
    type Event = MediaTimelineEvent;
    type Command = MediaTimelineCommand;

    fn initial(_input: &MediaTimelineInput) -> MediaTimelineState {
        MediaTimelineState::default()
    }

    fn transition(state: &MediaTimelineState, event: MediaTimelineEvent, input: &MediaTimelineInput) -> Step {
        use MediaTimelineEvent as E;
        let mut next = *state;
        match event {
            // Reports from the media element always apply.
            E::Time(t) => {
                if !state.scrubbing {
                    next.current = t.clamp(0.0, state.duration.max(t));
                }
            }
            E::Duration(d) => next.duration = if d.is_finite() && d > 0.0 { d } else { 0.0 },
            E::Buffered(b) => next.buffered = b.max(0.0),
            E::Playing(p) => next.playing = p,
            E::Hover(h) => next.hover = h.map(|t| t.clamp(0.0, state.duration)),
            _ if input.disabled => {}
            E::ScrubStart(t) => {
                next.scrubbing = true;
                return seek(next, t);
            }
            E::ScrubMove(t) if state.scrubbing => return seek(next, t),
            E::ScrubMove(_) => {}
            E::ScrubEnd => next.scrubbing = false,
            E::Nudge(secs) => return seek(next, state.current + secs),
            E::Page(f) => return seek(next, state.current + f * state.duration),
            E::ToStart => return seek(next, 0.0),
            E::ToEnd => return seek(next, state.duration),
            E::TogglePlay => {
                let command = if state.playing { MediaTimelineCommand::Pause } else { MediaTimelineCommand::Play };
                return Transition::with(next, vec![command]);
            }
        }
        Transition::to(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MediaTimelineEvent as E;

    fn run(events: &[E]) -> (MediaTimelineState, Vec<MediaTimelineCommand>) {
        let input = MediaTimelineInput::default();
        let mut s = MediaTimelineMachine::initial(&input);
        let mut all = Vec::new();
        for &e in events {
            let t = MediaTimelineMachine::transition(&s, e, &input).resolve(&s);
            s = t.state;
            all.extend(t.commands);
        }
        (s, all)
    }

    #[test]
    fn keys_seek_within_bounds() {
        let (s, cmds) = run(&[E::Duration(60.0), E::Nudge(-5.0), E::Nudge(5.0), E::Page(0.1), E::ToEnd, E::Nudge(5.0)]);
        assert!((s.current - 60.0).abs() < 1e-9);
        assert_eq!(cmds[0], MediaTimelineCommand::Seek(0.0));
        assert_eq!(cmds[1], MediaTimelineCommand::Seek(5.0));
        assert_eq!(cmds[2], MediaTimelineCommand::Seek(11.0));
    }

    #[test]
    fn scrubbing_ignores_media_time_until_released() {
        let (s, _) = run(&[E::Duration(100.0), E::ScrubStart(20.0), E::Time(3.0), E::ScrubMove(40.0)]);
        assert!((s.current - 40.0).abs() < 1e-9);
        let (s, _) = run(&[E::Duration(100.0), E::ScrubStart(20.0), E::ScrubEnd, E::Time(3.0)]);
        assert!((s.current - 3.0).abs() < 1e-9);
    }

    #[test]
    fn toggle_follows_playing_and_hover_clamps() {
        let (_, cmds) = run(&[E::TogglePlay, E::Playing(true), E::TogglePlay]);
        assert_eq!(cmds, [MediaTimelineCommand::Play, MediaTimelineCommand::Pause]);
        let (s, _) = run(&[E::Duration(10.0), E::Hover(Some(99.0))]);
        assert_eq!(s.hover, Some(10.0));
    }

    #[test]
    fn disabled_blocks_input_not_reports() {
        let input = MediaTimelineInput { disabled: true, ..Default::default() };
        let s = MediaTimelineMachine::initial(&input);
        assert!(MediaTimelineMachine::transition(&s, E::ToEnd, &input).resolve(&s).commands.is_empty());
        assert!((MediaTimelineMachine::transition(&s, E::Time(0.0), &input).resolve(&s).state.current).abs() < 1e-9);
    }
}
