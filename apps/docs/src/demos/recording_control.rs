use hydroxy_leptos::recording_control::{Phase, RecordingControl};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (phase, set_phase) = signal(Phase::Idle);

    view! {
        <div style="display: grid; gap: var(--space-5); max-width: 560px;">
            // The browser asks for the microphone on the first press. Time spent paused is not counted.
            <RecordingControl max_seconds=120 on_phase_change=move |p| set_phase.set(p) />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || format!("Phase: {}", phase.get().as_str())}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "recording-control",
    name: "Recording control",
    group: "Forms",
    summary: "Start, pause, resume and stop a microphone recording. Counts only the time spent recording, explains a refused or missing microphone, and plays the result back with a download link.",
    demo,
    leptos: include_str!("recording_control.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/recording_control.rs"),
    parts: &[
        &[
            "RecordingControl",
            "root",
            "role=group. data-state is idle, requesting, recording, paused, stopping, ready or error",
        ],
        &["(inside)", "buttons", "Wraps the three buttons"],
        &["(inside)", "start", "Starts a recording. Becomes Record again once one is ready"],
        &["(inside)", "pause", "Pauses, and resumes while paused. Hidden when no_pause is set"],
        &["(inside)", "stop", "Finishes the recording"],
        &["(inside)", "elapsed", "m:ss of recorded time"],
        &["(inside)", "status", "role=status text, including why recording is blocked"],
        &["(inside)", "result", "A player, a download link and a discard button for the finished recording"],
    ],
    keys: &[&["Enter / Space", "Press the focused button"]],
    props: &[
        &["max_seconds", "Option<u32>", "Stop on its own after this much recorded time"],
        &["no_pause", "bool", "Hide the pause button"],
        &["disabled", "bool", "Block recording"],
        &["label", "Option<String>", "Accessible name of the group"],
        &[
            "on_recorded",
            "Callback<Blob>",
            "The audio in the browser's native format (webm or mp4). Dioxus passes a RecordedAudio with an object URL, size and type instead",
        ],
        &["on_phase_change", "Callback<Phase>", "Called when the recorder changes phase"],
    ],
    events: &[],
};
