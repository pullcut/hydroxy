use dioxus::prelude::*;
use oxide_dioxus::recording_control::{Phase, RecordingControl};

#[component]
pub fn RecordingControlDemo() -> Element {
    let mut phase = use_signal(|| Phase::Idle);

    rsx! {
        div { style: "display: grid; gap: var(--space-5); max-width: 560px;",
            // The webview asks for the microphone on the first press. Time spent paused is not counted.
            RecordingControl { max_seconds: 120, on_phase_change: move |p| phase.set(p) }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Phase: {phase().as_str()}" }
        }
    }
}
