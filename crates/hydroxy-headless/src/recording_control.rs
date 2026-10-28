//! Recording control connector. Anatomy: root, start, pause, stop, elapsed,
//! status, result. Every part carries `data-scope="recording-control"` and
//! its `data-part`; the root's `data-state` is the recorder phase.

pub use hydroxy_machines::recording_control::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "recording-control";

/// Default accessible name for the group.
pub const LABEL: &str = "Recording";

/// Microphone icon markup for a 24x24 line-icon `<svg>`.
pub use crate::voice_input::{MIC, STOP};

/// Pause icon markup.
pub use crate::media_timeline::{PAUSE, PLAY};

/// The start button's accessible name.
#[must_use]
pub const fn start_label(phase: RecordingPhase) -> &'static str {
    match phase {
        RecordingPhase::Ready => "Record again",
        RecordingPhase::Error => "Try recording again",
        _ => "Start recording",
    }
}

/// The pause button's accessible name; it becomes resume while paused.
#[must_use]
pub const fn pause_label(phase: RecordingPhase) -> &'static str {
    match phase {
        RecordingPhase::Paused => "Resume recording",
        _ => "Pause recording",
    }
}

/// The stop button's accessible name.
pub const STOP_LABEL: &str = "Stop recording";

/// Milliseconds as `m:ss`.
#[must_use]
pub fn format_elapsed(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// The visible status line.
#[must_use]
pub fn status_text(state: &RecordingControlState) -> String {
    match state.phase {
        RecordingPhase::Idle => "Press to record".to_string(),
        RecordingPhase::Requesting => "Allow microphone access".to_string(),
        RecordingPhase::Recording => "Recording".to_string(),
        RecordingPhase::Paused => "Paused".to_string(),
        RecordingPhase::Stopping => "Finishing".to_string(),
        RecordingPhase::Ready => "Recording saved".to_string(),
        RecordingPhase::Error => state.error.map_or("Recording failed", RecordingError::message).to_string(),
    }
}

/// Where a finished recording file would be named: `recording-<id>.<ext>`.
#[must_use]
pub fn file_name(id: &str, mime_type: &str) -> String {
    let essence = mime_type.split(';').next().unwrap_or("").trim();
    let ext = match essence {
        "audio/webm" => "webm",
        "audio/ogg" => "ogg",
        "audio/mp4" | "audio/aac" => "m4a",
        "audio/mpeg" => "mp3",
        "audio/wav" | "audio/x-wav" => "wav",
        _ => "audio",
    };
    format!("recording-{id}.{ext}")
}

/// A byte count as `12 KB`.
#[must_use]
pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        #[allow(clippy::cast_precision_loss, reason = "display only")]
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{mb:.1} MB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_the_phase() {
        assert_eq!(pause_label(RecordingPhase::Paused), "Resume recording");
        assert_eq!(start_label(RecordingPhase::Ready), "Record again");
        assert_eq!(format_elapsed(65_000), "1:05");
    }

    #[test]
    fn status_explains_errors() {
        let s = RecordingControlState { phase: RecordingPhase::Error, error: Some(RecordingError::NoDevice), ..RecordingControlState::default() };
        assert_eq!(status_text(&s), "No microphone was found.");
    }

    #[test]
    fn names_files_by_type() {
        assert_eq!(file_name("7", "audio/webm;codecs=opus"), "recording-7.webm");
        assert_eq!(file_name("7", "audio/mp4"), "recording-7.m4a");
        assert_eq!(file_name("7", ""), "recording-7.audio");
        assert_eq!(format_size(2048), "2 KB");
    }
}
