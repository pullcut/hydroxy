//! Audio player connector. Anatomy: root, media, controls, play, seek, time,
//! mute, volume, rate, status, retry, transcript. Every part carries
//! `data-scope="audio-player"` and its `data-part`.

use crate::media_timeline::format_time;
pub use oxide_machines::audio_player::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "audio-player";

/// Play icon markup for a 24x24 line-icon `<svg>`.
pub use crate::media_timeline::{PAUSE, PLAY};

/// Replay (counter-clockwise arrow) icon markup.
pub const REPLAY: &str = r#"<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>"#;

/// Speaker icon markup.
pub const VOLUME: &str = r#"<path d="M11 5 6 9H2v6h4l5 4V5Z"/><path d="M15.5 8.5a5 5 0 0 1 0 7"/><path d="M19 5a10 10 0 0 1 0 14"/>"#;

/// Muted speaker icon markup.
pub const MUTED: &str = r#"<path d="M11 5 6 9H2v6h4l5 4V5Z"/><line x1="22" x2="16" y1="9" y2="15"/><line x1="16" x2="22" y1="9" y2="15"/>"#;

/// The play button's accessible name.
#[must_use]
pub fn play_label(status: PlayerStatus, playing: bool) -> &'static str {
    if playing {
        "Pause"
    } else if status == PlayerStatus::Ended {
        "Replay"
    } else {
        "Play"
    }
}

/// The mute button's accessible name.
#[must_use]
pub const fn mute_label(muted: bool) -> &'static str {
    if muted { "Unmute" } else { "Mute" }
}

/// The visible status line.
#[must_use]
pub fn status_text(status: PlayerStatus, error_code: u8) -> String {
    match status {
        PlayerStatus::Idle => "Preparing player",
        PlayerStatus::Loading => "Loading audio",
        PlayerStatus::Ready => "Ready to play",
        PlayerStatus::Playing => "Playing",
        PlayerStatus::Buffering => "Buffering",
        PlayerStatus::Paused => "Paused",
        PlayerStatus::Ended => "Finished",
        PlayerStatus::Blocked => "Playback was blocked by the browser. Press play to start.",
        PlayerStatus::Error => return error_message(error_code).to_string(),
    }
    .to_string()
}

/// A sentence for a `MediaError.code`.
#[must_use]
pub const fn error_message(code: u8) -> &'static str {
    match code {
        1 => "Loading was stopped before the audio was ready.",
        2 => "A network error interrupted loading.",
        3 => "The audio could not be decoded.",
        4 => "This source is not supported or has expired.",
        _ => "The audio could not be loaded.",
    }
}

/// `elapsed / total`, with `--:--` for an unknown total.
#[must_use]
pub fn time_text(current: f64, duration: Option<f64>) -> String {
    format!("{} / {}", format_time(current), duration.map_or_else(|| "--:--".to_string(), format_time))
}

/// The seek slider's `aria-valuetext`.
#[must_use]
pub fn seek_value_text(current: f64, duration: Option<f64>) -> String {
    match duration {
        Some(d) => format!("{} of {}", format_time(current), format_time(d)),
        None => format!("{}, duration unknown", format_time(current)),
    }
}

/// `1×`, `1.25×`.
#[must_use]
pub fn rate_text(rate: f64) -> String {
    let rounded = (rate * 100.0).round() / 100.0;
    format!("{rounded}\u{d7}")
}

/// Which part has focus, so native slider and button keys are not handled twice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Target {
    /// The root or a non-control.
    #[default]
    Other,
    /// A range input.
    Slider,
    /// A button.
    Button,
}

impl Target {
    /// Classify a `data-part` value.
    #[must_use]
    pub fn of(part: Option<&str>) -> Self {
        match part {
            Some("seek" | "volume") => Target::Slider,
            Some("play" | "mute" | "rate" | "retry" | "transcript") => Target::Button,
            _ => Target::Other,
        }
    }
}

/// The shared keyboard map (media-player conventions). Range inputs keep their
/// own arrow, Home and End keys; buttons keep Space and Enter. `None` when the
/// key is not a shortcut.
#[must_use]
pub fn resolve_key(key: &str, target: Target, input: &AudioPlayerInput) -> Option<AudioPlayerEvent> {
    let slider = target == Target::Slider;
    let step = input.seek_step;
    Some(match key {
        " " | "Spacebar" if target == Target::Other => AudioPlayerEvent::Toggle,
        "k" | "K" => AudioPlayerEvent::Toggle,
        "ArrowLeft" if !slider => AudioPlayerEvent::SeekBy(-step),
        "ArrowRight" if !slider => AudioPlayerEvent::SeekBy(step),
        "j" | "J" => AudioPlayerEvent::SeekBy(-step * 2.0),
        "l" | "L" => AudioPlayerEvent::SeekBy(step * 2.0),
        "Home" if !slider => AudioPlayerEvent::SeekFraction(0.0),
        "End" if !slider => AudioPlayerEvent::SeekFraction(1.0),
        "ArrowUp" if !slider => AudioPlayerEvent::VolumeBy(input.volume_step),
        "ArrowDown" if !slider => AudioPlayerEvent::VolumeBy(-input.volume_step),
        "m" | "M" => AudioPlayerEvent::ToggleMute,
        ">" => AudioPlayerEvent::CycleRate(1),
        "<" => AudioPlayerEvent::CycleRate(-1),
        _ => return None,
    })
}

/// Keys worth asking [`resolve_key`] about.
const CANDIDATE_KEYS: [&str; 15] =
    [" ", "k", "K", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "j", "J", "l", "L", "Home", "End", "m", ">"];

/// The keys the player claims when `part` has focus, so a script can stop the page
/// scrolling on them. Includes `<` as well as `>` and `M` as well as `m`.
#[must_use]
pub fn claimed_keys(part: Option<&str>, input: &AudioPlayerInput) -> Vec<&'static str> {
    let target = Target::of(part);
    let mut keys: Vec<&'static str> =
        CANDIDATE_KEYS.iter().chain(["M", "<"].iter()).copied().filter(|k| resolve_key(k, target, input).is_some()).collect();
    keys.dedup();
    keys
}

/// Controls are off while the source is broken.
#[must_use]
pub fn controls_disabled(status: PlayerStatus) -> bool {
    status == PlayerStatus::Error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_text() {
        assert_eq!(play_label(PlayerStatus::Ended, false), "Replay");
        assert_eq!(play_label(PlayerStatus::Playing, true), "Pause");
        assert_eq!(time_text(65.0, Some(200.0)), "1:05 / 3:20");
        assert_eq!(time_text(5.0, None), "0:05 / --:--");
        assert_eq!(seek_value_text(5.0, None), "0:05, duration unknown");
        assert_eq!(rate_text(1.0), "1\u{d7}");
        assert_eq!(rate_text(1.25), "1.25\u{d7}");
        assert_eq!(status_text(PlayerStatus::Error, 4), "This source is not supported or has expired.");
    }

    #[test]
    fn claimed_keys_follow_the_focused_part() {
        let input = AudioPlayerInput::default();
        let other = claimed_keys(None, &input);
        assert!(other.contains(&" ") && other.contains(&"ArrowUp"));
        let slider = claimed_keys(Some("seek"), &input);
        assert!(!slider.contains(&"ArrowUp") && !slider.contains(&" ") && slider.contains(&"k"));
    }

    #[test]
    fn keys_do_not_double_up_on_native_controls() {
        let input = AudioPlayerInput::default();
        assert_eq!(resolve_key(" ", Target::Other, &input), Some(AudioPlayerEvent::Toggle));
        assert_eq!(resolve_key(" ", Target::Button, &input), None);
        assert_eq!(resolve_key("ArrowRight", Target::Slider, &input), None);
        assert_eq!(resolve_key("ArrowRight", Target::Other, &input), Some(AudioPlayerEvent::SeekBy(5.0)));
        assert_eq!(resolve_key("j", Target::Slider, &input), Some(AudioPlayerEvent::SeekBy(-10.0)));
        assert_eq!(resolve_key(">", Target::Button, &input), Some(AudioPlayerEvent::CycleRate(1)));
        assert_eq!(resolve_key("x", Target::Other, &input), None);
        assert_eq!(Target::of(Some("volume")), Target::Slider);
    }
}
