use dioxus::prelude::*;
use hydroxy_dioxus::stat_ring::StatRing;

#[component]
pub fn StatRingDemo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-8);",
            StatRing { value: 7_420.0, goal: 10_000.0, label: "Weekly active" }
            StatRing { value: 38.0, goal: 40.0, label: "Story points" }
            StatRing { value: 1.12, goal: 1.0, label: "Quota", format: move |v: f64| format!("${v:.2}m") }
        }
    }
}
