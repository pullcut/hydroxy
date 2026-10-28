use dioxus::prelude::*;
use hydroxy_dioxus::step_track::{Jewel, StepTrack};

#[component]
pub fn StepTrackDemo() -> Element {
    let steps = || vec!["Draft".to_string(), "In review".into(), "Approved".into(), "Shared".into()];
    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            StepTrack { steps: steps(), current: Some(1), label: "Summary progress" }
            StepTrack { steps: steps(), current: Some(3), jewel: Jewel::Emerald, label: "Release progress" }
            StepTrack { steps: steps(), label: "Not started" }
        }
    }
}
