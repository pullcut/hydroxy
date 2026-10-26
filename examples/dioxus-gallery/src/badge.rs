use dioxus::prelude::*;
use oxide_dioxus::badge::{Badge, Tone};

#[component]
pub fn BadgeDemo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
            Badge { tone: Tone::Success, dot: true, "Ready" }
            Badge { tone: Tone::Warning, "2 flags" }
            Badge { tone: Tone::Danger, "Failed" }
            Badge { tone: Tone::Info, dot: true, "In review" }
            Badge { tone: Tone::Accent, "New" }
            Badge { "Draft" }
        }
    }
}
