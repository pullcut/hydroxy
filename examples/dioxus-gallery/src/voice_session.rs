use dioxus::prelude::*;
use hydroxy_dioxus::button::{Button, ButtonSize, ButtonVariant};
use hydroxy_dioxus::voice_session::{Phase, Speaker, TranscriptTurn, VoiceSession};

#[component]
pub fn VoiceSessionDemo() -> Element {
    // The owner holds the phase and reports what the transport is doing.
    let mut phase = use_signal(|| Phase::Idle);
    let mut turns = use_signal(Vec::<TranscriptTurn>::new);

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 480px;",
            VoiceSession {
                phase: Some(phase()),
                transcript: turns(),
                error_message: "The connection dropped. Check your network and try again.",
                on_start: move |()| turns.set(Vec::new()),
                on_phase_change: move |p| phase.set(p),
            }

            // Stand-ins for the audio transport, which is the owner's job.
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3); align-items: center;",
                span { style: "font: var(--text-sm); color: var(--text-2);", "Transport events:" }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    on_click: move |_| phase.set(Phase::Listening),
                    "Connected"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    on_click: move |_| {
                        turns.write().push(TranscriptTurn::new(Speaker::User, "What is on my calendar tomorrow?"));
                        phase.set(Phase::Speaking);
                    },
                    "Assistant speaks"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    on_click: move |_| {
                        turns
                            .write()
                            .push(TranscriptTurn::new(Speaker::Assistant, "You have a design review at ten and lunch with Ana at one."));
                        phase.set(Phase::Listening);
                    },
                    "Assistant stops"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    on_click: move |_| phase.set(Phase::Error),
                    "Drop the connection"
                }
            }
        }
    }
}
