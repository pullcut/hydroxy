//! Voice input connector. Anatomy: root, button, meter, bar, timer, status.
//! Every part carries `data-scope="voice-input"` and its `data-part`.

pub use hydroxy_machines::voice_input::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "voice-input";

/// Microphone icon markup for a 24x24 line-icon `<svg>` (Lucide, ISC licence).
pub const MIC: &str = r#"<path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" x2="12" y1="19" y2="22"/>"#;

/// Stop (square) icon markup.
pub const STOP: &str = r#"<rect x="6" y="6" width="12" height="12" rx="2"/>"#;

/// Number of bars in the level meter.
pub const BARS: usize = 5;

/// The button's accessible name.
#[must_use]
pub const fn button_label(phase: VoiceInputPhase) -> &'static str {
    match phase {
        VoiceInputPhase::Idle => "Start recording",
        VoiceInputPhase::Requesting => "Waiting for microphone",
        VoiceInputPhase::Recording => "Stop recording",
        VoiceInputPhase::Denied | VoiceInputPhase::Failed => "Try recording again",
    }
}

/// The visible status line.
#[must_use]
pub fn status_text(phase: VoiceInputPhase, elapsed_ms: u64) -> String {
    match phase {
        VoiceInputPhase::Idle => "Press to record".to_string(),
        VoiceInputPhase::Requesting => "Allow microphone access".to_string(),
        VoiceInputPhase::Recording => format!("Recording {}", format_elapsed(elapsed_ms)),
        VoiceInputPhase::Denied => "Microphone access was blocked. Allow it in your browser settings.".to_string(),
        VoiceInputPhase::Failed => "Recording is not available here.".to_string(),
    }
}

/// Milliseconds as `m:ss`.
#[must_use]
pub fn format_elapsed(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// The scale (0.15 to 1) for meter bar `i`, given the level. Bars in the middle move most.
#[must_use]
pub fn bar_scale(i: usize, level: f32) -> f32 {
    #[allow(clippy::cast_precision_loss, reason = "i is below BARS")]
    let weight = 1.0 - ((i as f32) - (BARS as f32 - 1.0) / 2.0).abs() / (BARS as f32);
    (0.15 + level.clamp(0.0, 1.0) * weight * 0.85 * 1.3).min(1.0)
}

/// Whether the button should be a pressed toggle.
#[must_use]
pub fn pressed(phase: VoiceInputPhase) -> bool {
    phase == VoiceInputPhase::Recording
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_time() {
        assert_eq!(button_label(VoiceInputPhase::Recording), "Stop recording");
        assert_eq!(format_elapsed(65_000), "1:05");
        assert_eq!(status_text(VoiceInputPhase::Recording, 7_400), "Recording 0:07");
    }

    #[test]
    fn meter_bars_grow_with_level() {
        assert!(bar_scale(2, 1.0) > bar_scale(2, 0.0));
        assert!(bar_scale(2, 0.5) > bar_scale(0, 0.5));
        assert!(bar_scale(2, 5.0) <= 1.0);
    }
}
