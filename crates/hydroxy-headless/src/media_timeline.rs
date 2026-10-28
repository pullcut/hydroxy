//! Media timeline connector. Anatomy: root, media, play, current, track,
//! buffered, played, marker, thumb, preview, duration, chapter. Every part
//! carries `data-scope="media-timeline"` and its `data-part`.

pub use hydroxy_machines::media_timeline::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "media-timeline";

/// Play icon markup for a 24x24 line-icon `<svg>`.
pub const PLAY: &str = r#"<polygon points="6 3 20 12 6 21 6 3"/>"#;

/// Pause icon markup.
pub const PAUSE: &str =
    r#"<rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/>"#;

/// Audio or video.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MediaKind {
    /// An `<audio>` element.
    #[default]
    Audio,
    /// A `<video>` element.
    Video,
}

/// A chapter or point of interest on the timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    /// Seconds from the start.
    pub time: f64,
    /// What starts here.
    pub label: String,
}

impl Marker {
    /// A marker at `time` seconds.
    #[must_use]
    pub fn new(time: f64, label: impl Into<String>) -> Self {
        Self { time, label: label.into() }
    }
}

/// Seconds as `m:ss`, or `h:mm:ss` from an hour up.
#[must_use]
pub fn format_time(seconds: f64) -> String {
    let total = if seconds.is_finite() && seconds > 0.0 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "non-negative and far below u64::MAX"
        )]
        let t = seconds as u64;
        t
    } else {
        0
    };
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

/// `aria-valuetext` for the thumb: "1:05 of 3:20".
#[must_use]
pub fn value_text(current: f64, duration: f64) -> String {
    format!("{} of {}", format_time(current), format_time(duration))
}

/// The time at a pointer position: `fraction` (0 to 1) along the track.
#[must_use]
pub fn time_at(fraction: f64, duration: f64) -> f64 {
    fraction.clamp(0.0, 1.0) * duration.max(0.0)
}

/// Index of the marker in effect at `time`: the last one at or before it.
#[must_use]
pub fn chapter_at(markers: &[Marker], time: f64) -> Option<usize> {
    markers
        .iter()
        .enumerate()
        .filter(|(_, m)| m.time <= time)
        .max_by(|a, b| a.1.time.total_cmp(&b.1.time))
        .map(|(i, _)| i)
}

/// Map a key on the thumb to a machine event. `Some` means: prevent the default.
/// Arrows move by the step (a fifth of it with Shift), `PageUp` and `PageDown` by a tenth of the duration.
#[must_use]
pub fn keydown(key: &str, shift: bool, step: f64) -> Option<MediaTimelineEvent> {
    let step = if shift { step / 5.0 } else { step };
    match key {
        "ArrowLeft" | "ArrowDown" => Some(MediaTimelineEvent::Nudge(-step)),
        "ArrowRight" | "ArrowUp" => Some(MediaTimelineEvent::Nudge(step)),
        "PageDown" => Some(MediaTimelineEvent::Page(-0.1)),
        "PageUp" => Some(MediaTimelineEvent::Page(0.1)),
        "Home" => Some(MediaTimelineEvent::ToStart),
        "End" => Some(MediaTimelineEvent::ToEnd),
        " " | "k" => Some(MediaTimelineEvent::TogglePlay),
        _ => None,
    }
}

/// The play button's accessible name.
#[must_use]
pub const fn play_label(playing: bool) -> &'static str {
    if playing { "Pause" } else { "Play" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_format() {
        assert_eq!(format_time(65.9), "1:05");
        assert_eq!(format_time(3725.0), "1:02:05");
        assert_eq!(format_time(f64::NAN), "0:00");
        assert_eq!(value_text(5.0, 200.0), "0:05 of 3:20");
    }

    #[test]
    fn chapters_and_keys() {
        let m = [Marker::new(0.0, "Intro"), Marker::new(30.0, "Demo"), Marker::new(90.0, "Q&A")];
        assert_eq!(chapter_at(&m, 45.0), Some(1));
        assert_eq!(chapter_at(&m, 90.0), Some(2));
        assert_eq!(chapter_at(&[Marker::new(10.0, "x")], 5.0), None);
        assert_eq!(keydown("ArrowRight", true, 5.0), Some(MediaTimelineEvent::Nudge(1.0)));
        assert_eq!(keydown("a", false, 5.0), None);
        assert!((time_at(0.5, 80.0) - 40.0).abs() < 1e-9);
    }
}
