use dioxus::prelude::*;
use hydroxy_dioxus::voice_input::{RecordedAudio, VoiceInput};

#[component]
pub fn VoiceInputDemo() -> Element {
    let mut clip = use_signal(|| None::<RecordedAudio>);

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 480px;",
            VoiceInput { max_seconds: 60, on_recorded: move |audio| clip.set(Some(audio)) }
            if let Some(audio) = clip() {
                // The audio arrives as an object URL. Fetch it for the bytes in a real app.
                audio { controls: true, src: "{audio.url}", style: "width: 100%;" }
                p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                    "{audio.size} bytes of {audio.mime_type}, {audio.duration_ms} ms"
                }
            }
        }
    }
}
