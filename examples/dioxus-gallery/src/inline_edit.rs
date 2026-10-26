use dioxus::prelude::*;
use oxide_dioxus::inline_edit::InlineEdit;

#[component]
pub fn InlineEditDemo() -> Element {
    let mut name = use_signal(|| String::from("Q3 customer interviews"));

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 420px;",
            div { style: "display: grid; gap: var(--space-2);",
                InlineEdit {
                    label: "Project name",
                    value: Some(name()),
                    on_change: move |v| name.set(v),
                }
                p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);", "Saved as: {name}" }
            }
            InlineEdit { label: "Description", placeholder: "Add a description" }
            InlineEdit {
                label: "Notes",
                multiline: true,
                default_value: "Interviews run Monday to Thursday.\nSynthesis on Friday.",
            }
            InlineEdit { label: "Locked title", default_value: "Archived project", disabled: true }
        }
    }
}
