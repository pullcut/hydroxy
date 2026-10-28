//! `VoiceSession`: the screen for a live voice conversation.

use dioxus::prelude::*;
use hydroxy_headless::voice_session::{
    self as h, Turn, VoicePhase, VoiceSessionCommand, VoiceSessionEvent, VoiceSessionInput, VoiceSessionMachine,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonVariant};

pub use h::{Speaker, Turn as TranscriptTurn, VoicePhase as Phase};

#[component]
fn Glyph(markup: &'static str) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            width: "20",
            height: "20",
            dangerous_inner_html: markup,
        }
    }
}

/// A voice conversation: status, mute and end controls, and a live transcript.
/// The audio transport is the owner's job. Report what it is doing through
/// `phase`; start and end it from `on_start` and `on_end`.
#[component]
pub fn VoiceSession(
    /// Controlled phase. Pair with `on_phase_change`, and drive it from the transport.
    #[props(default)]
    phase: ReadSignal<Option<VoicePhase>>,
    /// Initial phase when uncontrolled.
    #[props(default)]
    default_phase: VoicePhase,
    /// Controlled mute. Pair with `on_mute_change`.
    #[props(default)]
    muted: ReadSignal<Option<bool>>,
    /// Initial mute when uncontrolled.
    #[props(default)]
    default_muted: bool,
    /// The transcript, oldest first. A turn with `partial` set is still arriving.
    #[props(default)]
    transcript: Vec<Turn>,
    /// What went wrong, shown while the phase is `Error`.
    error_message: Option<String>,
    /// The user asked to start or retry. Open the transport.
    on_start: Option<EventHandler<()>>,
    /// The user ended or cancelled. Close the transport.
    on_end: Option<EventHandler<()>>,
    /// Called with the new mute flag. Apply it to the microphone track.
    on_mute_change: Option<EventHandler<bool>>,
    /// Called with the new phase whenever it changes or a change is requested.
    on_phase_change: Option<EventHandler<VoicePhase>>,
) -> Element {
    let input = use_memo(move || VoiceSessionInput {
        phase: phase(),
        default_phase,
        muted: muted(),
        default_muted,
    });
    let mut state = use_signal(|| VoiceSessionMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = VoiceSessionMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let mut send = move |event: VoiceSessionEvent| {
        let step = VoiceSessionMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                VoiceSessionCommand::Start => {
                    if let Some(h) = on_start {
                        h.call(());
                    }
                }
                VoiceSessionCommand::End => {
                    if let Some(h) = on_end {
                        h.call(());
                    }
                }
                VoiceSessionCommand::PhaseChange(p) => {
                    if let Some(h) = on_phase_change {
                        h.call(p);
                    }
                }
                VoiceSessionCommand::MuteChange(m) => {
                    if let Some(h) = on_mute_change {
                        h.call(m);
                    }
                }
            }
        }
    };

    let s = *state.read();
    let status = match (s.phase, &error_message) {
        (VoicePhase::Error, Some(message)) => message.clone(),
        (p, _) => h::status_text(p, s.muted).to_string(),
    };
    let idle = matches!(s.phase, VoicePhase::Idle | VoicePhase::Error);

    rsx! {
        section {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-state": s.phase.as_str(),
            "aria-label": "Voice conversation",
            div {
                "data-scope": h::SCOPE,
                "data-part": "orb",
                "data-state": s.phase.as_str(),
                "data-muted": s.muted.then_some(""),
                "aria-hidden": "true",
                span {}
            }
            p { "data-scope": h::SCOPE, "data-part": "status", role: "status", "{status}" }
            div { "data-scope": h::SCOPE, "data-part": "controls",
                if idle {
                    Button { variant: ButtonVariant::Accent, on_click: move |_| send(VoiceSessionEvent::Start), {h::start_label(s.phase)} }
                } else {
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "mute",
                        "aria-pressed": s.muted.to_string(),
                        "aria-label": h::mute_label(s.muted),
                        title: h::mute_label(s.muted),
                        disabled: !s.phase.is_live(),
                        onclick: move |_| send(VoiceSessionEvent::ToggleMute),
                        Glyph { markup: if s.muted { h::MIC_OFF } else { h::MIC } }
                    }
                    Button { variant: ButtonVariant::Danger, on_click: move |_| send(VoiceSessionEvent::End), {h::end_label(s.phase)} }
                }
            }
            ol {
                "data-scope": h::SCOPE,
                "data-part": "transcript",
                role: "log",
                "aria-label": "Transcript",
                "aria-live": "polite",
                for (i, turn) in transcript.iter().enumerate() {
                    li {
                        key: "{i}",
                        "data-scope": h::SCOPE,
                        "data-part": "turn",
                        "data-speaker": turn.speaker.as_str(),
                        "data-partial": turn.partial.then_some(""),
                        span { "data-scope": h::SCOPE, "data-part": "speaker", {h::speaker_label(turn.speaker)} }
                        span { "data-scope": h::SCOPE, "data-part": "text", "{turn.text}" }
                    }
                }
            }
        }
    }
}
