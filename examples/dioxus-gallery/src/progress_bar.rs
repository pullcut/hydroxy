use dioxus::prelude::*;
use oxide_dioxus::progress_bar::{Jewel, ProgressBar, ProgressSize};

#[component]
pub fn ProgressBarDemo() -> Element {
    let mut read = use_signal(|| 22.0);

    rsx! {
        div { style: "display: grid; gap: var(--space-7);",
            ProgressBar {
                label: "Reading interviews",
                value: Some(read()),
                max: 24.0,
                value_text: Some(format!("{} of 24", read())),
            }
            input {
                r#type: "range",
                min: "0",
                max: "24",
                value: "{read}",
                oninput: move |e| read.set(e.value().parse().unwrap_or(0.0)),
                "aria-label": "Interviews read",
                style: "accent-color: var(--accent);",
            }
            ProgressBar { label: "Storage", value: Some(64.0), jewel: Jewel::Emerald, size: ProgressSize::Sm }
            ProgressBar { label: "Drafting summary" }
        }
    }
}
