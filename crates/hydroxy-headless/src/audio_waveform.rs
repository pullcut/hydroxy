//! Audio waveform connector. Anatomy: root, track, bar, range, playhead,
//! range-start, range-end, loop, status. Every part carries
//! `data-scope="audio-waveform"` and its `data-part`.

use crate::media_timeline::format_time;
pub use hydroxy_machines::audio_waveform::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "audio-waveform";

/// Seconds an arrow key moves the playhead.
pub const SEEK_STEP: f64 = 1.0;

/// Seconds an arrow key moves a selection edge.
pub const RANGE_STEP: f64 = 0.25;

/// Default accessible name for the waveform.
pub const LABEL: &str = "Audio waveform";

/// A key on the track: the time it moves to, or `None` for other keys.
#[must_use]
pub fn track_key(key: &str, current: f64, duration: Option<f64>, step: f64) -> Option<f64> {
    match key {
        "ArrowLeft" | "ArrowDown" => Some(step_time(current, -step, duration)),
        "ArrowRight" | "ArrowUp" => Some(step_time(current, step, duration)),
        "Home" => Some(0.0),
        "End" => Some(clamp_time(duration.unwrap_or(0.0), duration)),
        _ => None,
    }
}

/// A key on a selection edge: the new range, or `None` for other keys.
#[must_use]
pub fn edge_key(key: &str, range: Range, edge: Edge, duration: Option<f64>, step: f64) -> Option<Range> {
    match key {
        "ArrowLeft" | "ArrowDown" => Some(step_edge(range, edge, -step, duration)),
        "ArrowRight" | "ArrowUp" => Some(step_edge(range, edge, step, duration)),
        _ => None,
    }
}

/// The `aria-valuetext` for the playhead: "0:05 of 3:20".
#[must_use]
pub fn value_text(current: f64, duration: Option<f64>) -> String {
    format!("{} of {}", format_time(current), duration.map_or_else(|| "--:--".to_string(), format_time))
}

/// The status line: position, selection and any input problems.
#[must_use]
pub fn status_text(p: &Projection) -> String {
    let mut text =
        format!("{} / {}", format_time(p.current), p.duration.map_or_else(|| "--:--".to_string(), format_time));
    if let Some(r) = p.range {
        text = format!("{text} \u{b7} selection {}\u{2013}{}", format_time(r.start), format_time(r.end));
    }
    for issue in &p.issues {
        text.push_str(" \u{b7} ");
        text.push_str(issue.as_str());
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_move_the_playhead_and_edges() {
        assert_eq!(track_key("ArrowRight", 9.5, Some(10.0), 1.0), Some(10.0));
        assert_eq!(track_key("Home", 4.0, Some(10.0), 1.0), Some(0.0));
        assert_eq!(track_key("End", 4.0, Some(10.0), 1.0), Some(10.0));
        assert_eq!(track_key("x", 4.0, Some(10.0), 1.0), None);
        let r = Range { start: 1.0, end: 2.0 };
        assert_eq!(edge_key("ArrowRight", r, Edge::End, Some(10.0), 0.25), Some(Range { start: 1.0, end: 2.25 }));
    }

    #[test]
    fn status_lists_selection_and_issues() {
        let peaks = Peaks::new(vec![0.5; 4], 1000.0, 1000.0);
        let p = project(Some(&peaks), Some(4.0), 1.0, Some(Range { start: 1.0, end: 2.0 }), 1.0, 0.0);
        assert_eq!(status_text(&p), "0:01 / 0:04 \u{b7} selection 0:01\u{2013}0:02");
        let bad = project(None, Some(4.0), 0.0, None, 1.0, 0.0);
        assert!(status_text(&bad).ends_with("missing-peaks"));
        assert_eq!(value_text(5.0, None), "0:05 of --:--");
    }
}
