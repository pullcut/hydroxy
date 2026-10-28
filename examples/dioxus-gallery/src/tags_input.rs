use dioxus::prelude::*;
use hydroxy_dioxus::tags_input::TagsInput;

#[component]
pub fn TagsInputDemo() -> Element {
    let mut topics = use_signal(|| vec!["onboarding".to_string(), "billing".to_string()]);

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 480px;",
            // Controlled, with a limit and a form name.
            TagsInput {
                label: "Topics",
                value: Some(topics()),
                on_value_change: move |tags| topics.set(tags),
                max: 5,
                name: "topics",
                placeholder: "Add a topic",
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                "Tagged: {topics().join(\", \")}"
            }

            // Uncontrolled.
            TagsInput { label: "Notify these people", placeholder: "Type a name, then press Enter" }

            // Disabled.
            TagsInput {
                label: "Shared with",
                default_value: vec!["Jo Marsh".to_string(), "Ana Kowal".to_string()],
                disabled: true,
            }
        }
    }
}
