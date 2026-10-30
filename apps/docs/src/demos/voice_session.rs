use hydroxy_leptos::voice_session::{Phase, Speaker, TranscriptTurn, VoiceSession};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // The owner holds the phase and reports what the transport is doing.
    let (phase, set_phase) = signal(Phase::Idle);
    let (turns, set_turns) = signal(Vec::<TranscriptTurn>::new());

    let say = move |speaker, text: &str| set_turns.update(|t| t.push(TranscriptTurn::new(speaker, text)));

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 480px;">
            <VoiceSession
                phase=phase
                transcript=turns
                error_message="The connection dropped. Check your network and try again."
                on_start=move |()| set_turns.set(Vec::new())
                on_phase_change=move |p| set_phase.set(p)
            />

            // Stand-ins for the audio transport, which is the owner's job.
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-3); align-items: center;">
                <span style="font: var(--text-sm); color: var(--text-2);">"Transport events:"</span>
                <hydroxy_leptos::button::Button
                    variant=hydroxy_leptos::button::ButtonVariant::Secondary
                    size=hydroxy_leptos::button::ButtonSize::Sm
                    on_click=move |_| set_phase.set(Phase::Listening)
                >
                    "Connected"
                </hydroxy_leptos::button::Button>
                <hydroxy_leptos::button::Button
                    variant=hydroxy_leptos::button::ButtonVariant::Secondary
                    size=hydroxy_leptos::button::ButtonSize::Sm
                    on_click=move |_| {
                        say(Speaker::User, "What is on my calendar tomorrow?");
                        set_phase.set(Phase::Speaking);
                    }
                >
                    "Assistant speaks"
                </hydroxy_leptos::button::Button>
                <hydroxy_leptos::button::Button
                    variant=hydroxy_leptos::button::ButtonVariant::Secondary
                    size=hydroxy_leptos::button::ButtonSize::Sm
                    on_click=move |_| {
                        say(Speaker::Assistant, "You have a design review at ten and lunch with Ana at one.");
                        set_phase.set(Phase::Listening);
                    }
                >
                    "Assistant stops"
                </hydroxy_leptos::button::Button>
                <hydroxy_leptos::button::Button
                    variant=hydroxy_leptos::button::ButtonVariant::Secondary
                    size=hydroxy_leptos::button::ButtonSize::Sm
                    on_click=move |_| set_phase.set(Phase::Error)
                >
                    "Drop the connection"
                </hydroxy_leptos::button::Button>
            </div>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "voice-session",
    name: "Voice session",
    group: "Chat",
    summary: "The screen for a live voice conversation: a status orb, mute and end controls and a transcript. The audio transport is yours; the component takes the phase and reports what the user asked for.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("voice_session.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/voice_session.rs"),
    parts: &[
        &["VoiceSession", "root", "A labelled section. data-state is idle, connecting, listening, speaking or error"],
        &["(inside)", "orb", "Decorative. Rings and pulses change with the phase; data-muted while muted"],
        &["(inside)", "status", "role=status text for the phase, or the error message"],
        &["(inside)", "controls", "Start button, or mute and end buttons while a call is up"],
        &["(inside)", "mute", "Toggle button with aria-pressed, enabled while listening or speaking"],
        &["(inside)", "transcript", "role=log list that announces new lines politely"],
        &["(inside)", "turn", "One line. data-speaker is user or assistant; data-partial while it is still arriving"],
    ],
    keys: &[&["Tab", "Move between the mute, end and start buttons"], &["Enter / Space", "Press the focused button"]],
    props: &[
        &[
            "phase",
            "Option<Phase>",
            "Controlled phase: Idle, Connecting, Listening, Speaking or Error. Drive it from the transport",
        ],
        &["default_phase", "Phase", "Initial phase when uncontrolled"],
        &["muted", "Option<bool>", "Controlled mute. Pair with on_mute_change"],
        &["default_muted", "bool", "Initial mute when uncontrolled"],
        &["transcript", "Vec<TranscriptTurn>", "Oldest first. A partial turn shows an ellipsis"],
        &["error_message", "String", "Shown instead of the status while the phase is Error"],
        &["on_start", "Callback<()>", "The user asked to start or retry. Open the transport"],
        &["on_end", "Callback<()>", "The user ended or cancelled. Close the transport"],
        &["on_mute_change", "Callback<bool>", "Apply the new flag to the microphone track"],
        &["on_phase_change", "Callback<Phase>", "The phase the user's action asks for. Set it on your signal"],
    ],
    events: &[],
};
