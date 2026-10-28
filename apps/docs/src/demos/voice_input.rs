use hydroxy_leptos::voice_input::VoiceInput;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (clip, set_clip) = signal(None::<String>);

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 480px;">
            <VoiceInput
                max_seconds=60
                on_recorded=move |blob: web_sys::Blob| {
                    // Play it back through an object URL. Upload the Blob instead in a real app.
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        set_clip.set(Some(url));
                    }
                }
            />
            {move || {
                clip.get()
                    .map(|url| {
                        view! { <audio controls=true src=url style="width: 100%;"></audio> }
                    })
            }}
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "voice-input",
    name: "Voice input",
    group: "Forms",
    summary: "A microphone button that records with getUserMedia and MediaRecorder, shows a live level meter, and hands you the audio as a Blob. Handles the permission prompt and a refusal.",
    demo,
    leptos: include_str!("voice_input.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/voice_input.rs"),
    parts: &[
        &["VoiceInput", "root", "Wraps everything. data-state is idle, requesting, recording, denied or failed"],
        &["(inside)", "button", "Round toggle with aria-pressed and a label that follows the state"],
        &["(inside)", "meter", "Five bars driven by an AnalyserNode. Decorative"],
        &["(inside)", "bar", "One meter bar, scaled by the input level"],
        &["(inside)", "status", "role=status text: prompt, elapsed time, or why recording is blocked"],
    ],
    keys: &[&["Enter / Space", "Start recording, or stop while recording"]],
    props: &[
        &["max_seconds", "Option<u32>", "Stop on its own after this long"],
        &["disabled", "bool", "Block recording"],
        &[
            "on_recorded",
            "Callback<Blob>",
            "The audio in the browser's native format (webm or mp4). Dioxus passes a RecordedAudio with an object URL, size and type instead",
        ],
        &[
            "on_phase_change",
            "Callback<VoiceInputPhase>",
            "Called when the recorder moves between idle, requesting, recording, denied and failed",
        ],
    ],
};
