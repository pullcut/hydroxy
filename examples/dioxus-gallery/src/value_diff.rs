use dioxus::prelude::*;
use hydroxy_dioxus::value_diff::ValueDiff;

#[component]
pub fn ValueDiffDemo() -> Element {
    rsx! {
        ul { style: "list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);",
            li { style: "display: flex; align-items: center; gap: 10px; font: var(--text-sm);",
                "Jo Marsh moved the status"
                ValueDiff { from: "In review", to: "Ready" }
            }
            li { style: "display: flex; align-items: center; gap: 10px; font: var(--text-sm);",
                "Amara Kaur changed the due date"
                ValueDiff { from: "24 Oct", to: "31 Oct" }
            }
        }
    }
}
