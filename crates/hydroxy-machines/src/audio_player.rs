//! Audio player: the status, volume and rate rules for a native `<audio>` element.
//!
//! The media element owns the clock. The adapter reports what the element says
//! as a [`MediaFacts`] snapshot; the machine derives the visible status and turns
//! button and key input into `Play`, `Seek`, `Volume`, `Muted` and `Rate` commands.

use hydroxy_statechart::{Machine, Transition};

/// Playback rates the rate button cycles through.
pub const DEFAULT_RATES: [f64; 5] = [0.75, 1.0, 1.25, 1.5, 2.0];

/// Visible playback status, derived from the element plus two facts only the
/// session knows (a pending `play()` and a blocked attempt).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PlayerStatus {
    /// No media element yet.
    #[default]
    Idle,
    /// Waiting for metadata.
    Loading,
    /// Metadata is in, nothing played yet.
    Ready,
    /// Playing.
    Playing,
    /// Wants to play, waiting for data.
    Buffering,
    /// Paused part-way.
    Paused,
    /// Reached the end.
    Ended,
    /// The browser refused to autoplay. A press starts it.
    Blocked,
    /// The source failed.
    Error,
}

impl PlayerStatus {
    /// The `data-status` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PlayerStatus::Idle => "idle",
            PlayerStatus::Loading => "loading",
            PlayerStatus::Ready => "ready",
            PlayerStatus::Playing => "playing",
            PlayerStatus::Buffering => "buffering",
            PlayerStatus::Paused => "paused",
            PlayerStatus::Ended => "ended",
            PlayerStatus::Blocked => "blocked",
            PlayerStatus::Error => "error",
        }
    }
}

/// What the media element reports. Zero or `NaN` duration means unknown.
#[allow(clippy::struct_excessive_bools, reason = "a mirror of the media element's own flags")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaFacts {
    /// `currentTime`.
    pub current: f64,
    /// `duration`.
    pub duration: f64,
    /// `paused`.
    pub paused: bool,
    /// `ended`.
    pub ended: bool,
    /// `seeking`.
    pub seeking: bool,
    /// `readyState`, 0 to 4.
    pub ready_state: u8,
    /// `error.code`, or 0 for none.
    pub error_code: u8,
    /// `volume`, 0 to 1.
    pub volume: f64,
    /// `muted`.
    pub muted: bool,
    /// `playbackRate`.
    pub rate: f64,
}

impl Default for MediaFacts {
    fn default() -> Self {
        Self {
            current: 0.0,
            duration: 0.0,
            paused: true,
            ended: false,
            seeking: false,
            ready_state: 0,
            error_code: 0,
            volume: 1.0,
            muted: false,
            rate: 1.0,
        }
    }
}

/// Props for [`AudioPlayerMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct AudioPlayerInput {
    /// Seconds a skip moves.
    pub seek_step: f64,
    /// Volume a step moves, 0 to 1.
    pub volume_step: f64,
    /// Rates the rate control cycles, ascending.
    pub rates: Vec<f64>,
}

impl Default for AudioPlayerInput {
    fn default() -> Self {
        Self { seek_step: 5.0, volume_step: 0.1, rates: DEFAULT_RATES.to_vec() }
    }
}

/// What [`AudioPlayerMachine`] remembers between events.
#[allow(clippy::struct_excessive_bools, reason = "independent session flags")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudioPlayerState {
    /// The latest element snapshot.
    pub facts: MediaFacts,
    /// `play()` was asked for and has not settled.
    pub pending_play: bool,
    /// The last `play()` was refused.
    pub blocked: bool,
    /// Playback has started at least once for this load.
    pub started: bool,
    /// A media element is attached.
    pub attached: bool,
}

impl AudioPlayerState {
    /// The visible status.
    #[must_use]
    pub fn status(&self) -> PlayerStatus {
        let f = &self.facts;
        if !self.attached {
            return PlayerStatus::Idle;
        }
        if f.error_code != 0 {
            return PlayerStatus::Error;
        }
        if self.blocked {
            return PlayerStatus::Blocked;
        }
        if f.ended {
            return PlayerStatus::Ended;
        }
        if !f.paused || self.pending_play {
            return if f.ready_state >= 3 && !f.seeking { PlayerStatus::Playing } else { PlayerStatus::Buffering };
        }
        if f.ready_state < 1 {
            return PlayerStatus::Loading;
        }
        if f.current > 0.0 || f.seeking || self.started { PlayerStatus::Paused } else { PlayerStatus::Ready }
    }

    /// Playing or about to.
    #[must_use]
    pub fn playing(&self) -> bool {
        matches!(self.status(), PlayerStatus::Playing | PlayerStatus::Buffering)
    }

    /// The duration, when known.
    #[must_use]
    pub fn duration(&self) -> Option<f64> {
        (self.facts.duration.is_finite() && self.facts.duration > 0.0).then_some(self.facts.duration)
    }
}

/// Inputs to [`AudioPlayerMachine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioPlayerEvent {
    /// A media element is mounted.
    Attached,
    /// The element reported its state.
    Facts(MediaFacts),
    /// `play()` was rejected by the browser.
    PlayRejected,
    /// Play when paused, pause when playing, replay at the end.
    Toggle,
    /// Skip this many seconds (negative for back).
    SeekBy(f64),
    /// Jump to this fraction of the duration.
    SeekFraction(f64),
    /// Jump to this time.
    SeekTo(f64),
    /// Change the volume by this much.
    VolumeBy(f64),
    /// Set the volume.
    SetVolume(f64),
    /// Mute or unmute.
    ToggleMute,
    /// Next (1) or previous (-1) rate.
    CycleRate(i8),
    /// Load the source again after an error.
    Retry,
}

/// Side effects from [`AudioPlayerMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioPlayerCommand {
    /// Call `play()`; report [`AudioPlayerEvent::PlayRejected`] if it fails.
    Play,
    /// Call `pause()`.
    Pause,
    /// Set `currentTime`.
    Seek(f64),
    /// Set `volume`.
    Volume(f64),
    /// Set `muted`.
    Muted(bool),
    /// Set `playbackRate`.
    Rate(f64),
    /// Call `load()`.
    Reload,
}

/// Status, seek, volume and rate for a media element.
#[derive(Debug, Clone, Copy)]
pub struct AudioPlayerMachine;

type Step = Transition<AudioPlayerState, AudioPlayerCommand>;

/// Clamp a time to `0..=duration`. There is no upper bound while the duration is unknown,
/// which includes a zero, negative or non-finite one. Shared with `audio_waveform`.
#[must_use]
pub fn clamp_time(time: f64, duration: Option<f64>) -> f64 {
    let t = if time.is_finite() { time.max(0.0) } else { 0.0 };
    duration.filter(|d| d.is_finite() && *d > 0.0).map_or(t, |d| t.min(d))
}

/// Volume steps stay on a two-decimal grid so repeated moves land on exact 0 and 1.
#[must_use]
pub fn step_volume(volume: f64, delta: f64) -> f64 {
    (((volume + delta) * 100.0).round() / 100.0).clamp(0.0, 1.0)
}

/// The next rate in the cycle. Wraps at either end; an unknown current rate snaps to its neighbour.
#[must_use]
pub fn next_rate(current: f64, rates: &[f64], direction: i8) -> f64 {
    if rates.is_empty() {
        return current;
    }
    let forward = direction >= 0;
    if let Some(i) = rates.iter().position(|r| (r - current).abs() < 1e-9) {
        let n = rates.len();
        return rates[if forward { (i + 1) % n } else { (i + n - 1) % n }];
    }
    let pick = if forward {
        rates.iter().copied().filter(|r| *r > current).fold(None, |m: Option<f64>, r| Some(m.map_or(r, |m| m.min(r))))
    } else {
        rates.iter().copied().filter(|r| *r < current).fold(None, |m: Option<f64>, r| Some(m.map_or(r, |m| m.max(r))))
    };
    pick.unwrap_or(if forward { rates[0] } else { rates[rates.len() - 1] })
}

impl Machine for AudioPlayerMachine {
    type Input = AudioPlayerInput;
    type State = AudioPlayerState;
    type Event = AudioPlayerEvent;
    type Command = AudioPlayerCommand;

    fn initial(_input: &AudioPlayerInput) -> AudioPlayerState {
        AudioPlayerState::default()
    }

    fn transition(state: &AudioPlayerState, event: AudioPlayerEvent, input: &AudioPlayerInput) -> Step {
        use AudioPlayerCommand as C;
        use AudioPlayerEvent as E;
        let mut next = state.clone();
        let mut commands = Vec::new();
        let duration = state.duration();
        match event {
            E::Attached => next.attached = true,
            E::Facts(facts) => {
                next.facts = facts;
                if !facts.paused {
                    next.pending_play = false;
                    next.blocked = false;
                    next.started = true;
                }
                if facts.error_code != 0 {
                    next.pending_play = false;
                }
            }
            E::PlayRejected => {
                next.pending_play = false;
                next.blocked = true;
            }
            E::Toggle => {
                if state.status() == PlayerStatus::Error {
                    return Transition::update(state, next, commands);
                }
                if state.playing() {
                    next.pending_play = false;
                    commands.push(C::Pause);
                } else {
                    next.pending_play = true;
                    next.blocked = false;
                    if state.facts.ended {
                        next.facts.current = 0.0;
                        commands.push(C::Seek(0.0));
                    }
                    commands.push(C::Play);
                }
            }
            E::SeekBy(delta) => {
                let to = clamp_time(state.facts.current + delta, duration);
                next.facts.current = to;
                commands.push(C::Seek(to));
            }
            E::SeekFraction(fraction) => {
                if let Some(d) = duration {
                    let to = clamp_time(fraction.clamp(0.0, 1.0) * d, duration);
                    next.facts.current = to;
                    commands.push(C::Seek(to));
                }
            }
            E::SeekTo(time) => {
                let to = clamp_time(time, duration);
                next.facts.current = to;
                commands.push(C::Seek(to));
            }
            E::VolumeBy(delta) => {
                let v = step_volume(state.facts.volume, delta);
                next.facts.volume = v;
                commands.push(C::Volume(v));
                if state.facts.muted && delta > 0.0 {
                    next.facts.muted = false;
                    commands.push(C::Muted(false));
                }
            }
            E::SetVolume(volume) => {
                let v = volume.clamp(0.0, 1.0);
                next.facts.volume = v;
                commands.push(C::Volume(v));
                if state.facts.muted && v > 0.0 {
                    next.facts.muted = false;
                    commands.push(C::Muted(false));
                }
            }
            E::ToggleMute => {
                next.facts.muted = !state.facts.muted;
                commands.push(C::Muted(next.facts.muted));
            }
            E::CycleRate(direction) => {
                let rate = next_rate(state.facts.rate, &input.rates, direction);
                next.facts.rate = rate;
                commands.push(C::Rate(rate));
            }
            E::Retry => {
                next.facts.error_code = 0;
                next.blocked = false;
                next.started = false;
                next.pending_play = false;
                commands.push(C::Reload);
            }
        }
        Transition::update(state, next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AudioPlayerCommand as C;
    use AudioPlayerEvent as E;
    use hydroxy_statechart::step as advance;

    #[test]
    fn clamp_time_bounds_only_by_a_known_duration() {
        assert!((clamp_time(15.0, Some(10.0)) - 10.0).abs() < f64::EPSILON);
        assert!(clamp_time(-3.0, Some(10.0)).abs() < f64::EPSILON);
        assert!((clamp_time(15.0, None) - 15.0).abs() < f64::EPSILON);
        assert!((clamp_time(15.0, Some(0.0)) - 15.0).abs() < f64::EPSILON);
        assert!(clamp_time(f64::NAN, Some(10.0)).abs() < f64::EPSILON);
    }

    fn send(s: &AudioPlayerState, e: E) -> hydroxy_statechart::Step<AudioPlayerState, AudioPlayerCommand> {
        advance::<AudioPlayerMachine>(s, e, &AudioPlayerInput::default())
    }

    fn loaded() -> AudioPlayerState {
        let mut s = AudioPlayerMachine::initial(&AudioPlayerInput::default());
        s = send(&s, E::Attached).state;
        send(&s, E::Facts(MediaFacts { duration: 100.0, ready_state: 4, ..MediaFacts::default() })).state
    }

    #[test]
    fn status_follows_the_element() {
        let mut s = AudioPlayerState::default();
        assert_eq!(s.status(), PlayerStatus::Idle);
        s = send(&s, E::Attached).state;
        assert_eq!(s.status(), PlayerStatus::Loading);
        s = loaded();
        assert_eq!(s.status(), PlayerStatus::Ready);
        let t = send(&s, E::Toggle);
        assert_eq!(t.commands, [C::Play]);
        assert_eq!(t.state.status(), PlayerStatus::Playing);
        let playing = send(
            &t.state,
            E::Facts(MediaFacts {
                paused: false,
                current: 3.0,
                duration: 100.0,
                ready_state: 4,
                ..MediaFacts::default()
            }),
        );
        assert_eq!(playing.state.status(), PlayerStatus::Playing);
        let paused = send(&playing.state, E::Toggle);
        assert_eq!(paused.commands, [C::Pause]);
    }

    #[test]
    fn rejected_play_blocks_until_pressed() {
        let s = loaded();
        let s = send(&s, E::Toggle).state;
        let s = send(&s, E::PlayRejected).state;
        assert_eq!(s.status(), PlayerStatus::Blocked);
        assert_eq!(send(&s, E::Toggle).commands, [C::Play]);
    }

    #[test]
    fn errors_stop_everything_until_retry() {
        let s = send(&loaded(), E::Facts(MediaFacts { error_code: 4, ..MediaFacts::default() })).state;
        assert_eq!(s.status(), PlayerStatus::Error);
        assert!(send(&s, E::Toggle).commands.is_empty());
        let r = send(&s, E::Retry);
        assert_eq!(r.commands, [C::Reload]);
    }

    #[test]
    fn replay_from_the_end() {
        let s = send(
            &loaded(),
            E::Facts(MediaFacts {
                ended: true,
                current: 100.0,
                duration: 100.0,
                ready_state: 4,
                ..MediaFacts::default()
            }),
        )
        .state;
        assert_eq!(s.status(), PlayerStatus::Ended);
        assert_eq!(send(&s, E::Toggle).commands, [C::Seek(0.0), C::Play]);
    }

    #[test]
    fn seeking_clamps() {
        let s = loaded();
        assert_eq!(send(&s, E::SeekBy(-5.0)).commands, [C::Seek(0.0)]);
        assert_eq!(send(&s, E::SeekTo(500.0)).commands, [C::Seek(100.0)]);
        assert_eq!(send(&s, E::SeekFraction(0.25)).commands, [C::Seek(25.0)]);
        let unknown = send(&AudioPlayerState::default(), E::SeekFraction(0.5));
        assert!(unknown.commands.is_empty());
    }

    #[test]
    fn volume_and_mute() {
        let mut s = loaded();
        s.facts.volume = 0.95;
        assert_eq!(send(&s, E::VolumeBy(0.1)).commands, [C::Volume(1.0)]);
        s.facts.muted = true;
        assert_eq!(send(&s, E::SetVolume(0.4)).commands, [C::Volume(0.4), C::Muted(false)]);
        assert_eq!(send(&s, E::ToggleMute).commands, [C::Muted(false)]);
    }

    #[test]
    fn rates_cycle_and_snap() {
        let r = DEFAULT_RATES;
        assert!((next_rate(1.0, &r, 1) - 1.25).abs() < 1e-9);
        assert!((next_rate(2.0, &r, 1) - 0.75).abs() < 1e-9);
        assert!((next_rate(0.75, &r, -1) - 2.0).abs() < 1e-9);
        assert!((next_rate(1.1, &r, 1) - 1.25).abs() < 1e-9);
        assert!((next_rate(1.1, &r, -1) - 1.0).abs() < 1e-9);
    }
}
