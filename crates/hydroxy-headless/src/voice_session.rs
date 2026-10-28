//! Voice session connector. Anatomy: root, status, orb, controls, mute, end,
//! start, transcript, turn, live. Every part carries `data-scope="voice-session"`
//! and its `data-part`.

pub use hydroxy_machines::voice_session::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "voice-session";

/// Microphone icon markup for a 24x24 line-icon `<svg>`.
pub const MIC: &str = crate::voice_input::MIC;

/// Muted microphone icon markup.
pub const MIC_OFF: &str = r#"<line x1="2" x2="22" y1="2" y2="22"/><path d="M18.89 13.23A7.12 7.12 0 0 0 19 12v-2"/><path d="M5 10v2a7 7 0 0 0 12 5"/><path d="M15 9.34V5a3 3 0 0 0-5.68-1.33"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12"/><line x1="12" x2="12" y1="19" y2="22"/>"#;

/// Hang-up icon markup.
pub const PHONE_OFF: &str = r#"<path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-3.33-2.67m-2.67-3.34a19.79 19.79 0 0 1-3.07-8.63A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91"/><line x1="22" x2="2" y1="2" y2="22"/>"#;

/// Who said a line of the transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    /// The person on the call.
    User,
    /// The assistant.
    Assistant,
}

impl Speaker {
    /// The `data-speaker` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Speaker::User => "user",
            Speaker::Assistant => "assistant",
        }
    }
}

/// One line of the transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    /// Who spoke.
    pub speaker: Speaker,
    /// What was said so far.
    pub text: String,
    /// Still being spoken: more text may arrive.
    pub partial: bool,
}

impl Turn {
    /// A finished line.
    #[must_use]
    pub fn new(speaker: Speaker, text: impl Into<String>) -> Self {
        Self { speaker, text: text.into(), partial: false }
    }

    /// Mark as still arriving.
    #[must_use]
    pub fn partial(mut self) -> Self {
        self.partial = true;
        self
    }
}

/// The visible status line.
#[must_use]
pub const fn status_text(phase: VoicePhase, muted: bool) -> &'static str {
    match phase {
        VoicePhase::Idle => "Ready to talk",
        VoicePhase::Connecting => "Connecting",
        VoicePhase::Listening if muted => "Muted",
        VoicePhase::Listening => "Listening",
        VoicePhase::Speaking => "Speaking",
        VoicePhase::Error => "Connection lost",
    }
}

/// The mute button's accessible name.
#[must_use]
pub const fn mute_label(muted: bool) -> &'static str {
    if muted { "Unmute microphone" } else { "Mute microphone" }
}

/// The start button's text.
#[must_use]
pub const fn start_label(phase: VoicePhase) -> &'static str {
    if matches!(phase, VoicePhase::Error) { "Try again" } else { "Start conversation" }
}

/// The end button's text: cancelling while connecting, ending when live.
#[must_use]
pub const fn end_label(phase: VoicePhase) -> &'static str {
    if matches!(phase, VoicePhase::Connecting) { "Cancel" } else { "End conversation" }
}

/// The transcript speaker's visible name.
#[must_use]
pub const fn speaker_label(speaker: Speaker) -> &'static str {
    match speaker {
        Speaker::User => "You",
        Speaker::Assistant => "Assistant",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_the_phase() {
        assert_eq!(status_text(VoicePhase::Listening, true), "Muted");
        assert_eq!(status_text(VoicePhase::Speaking, true), "Speaking");
        assert_eq!(end_label(VoicePhase::Connecting), "Cancel");
        assert_eq!(start_label(VoicePhase::Error), "Try again");
        assert_eq!(mute_label(true), "Unmute microphone");
    }
}
