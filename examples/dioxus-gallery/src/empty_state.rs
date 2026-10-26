use dioxus::prelude::*;
use oxide_dioxus::empty_state::EmptyState;
use oxide_dioxus::icon::IconName;

#[component]
pub fn EmptyStateDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            EmptyState {
                title: "Nothing archived yet",
                description: "Archived projects show up here. Archiving keeps their files and summaries.",
                a { href: "#", "Open the project list" }
            }
            div { style: "border: 1px dashed var(--border-2); border-radius: var(--radius-lg);",
                EmptyState {
                    icon: IconName::Search,
                    title: "No matches",
                    description: "Nothing in this workspace mentions \"refund policy\".",
                }
            }
        }
    }
}
