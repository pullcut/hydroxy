use dioxus::prelude::*;
use oxide_dioxus::button::{Button, ButtonSize, ButtonVariant};
use oxide_dioxus::icon::IconName;

#[component]
pub fn ButtonDemo() -> Element {
    let mut drafts = use_signal(|| 0);

    rsx! {
        div { style: "display: grid; gap: var(--space-7);",
            // Variants. One primary or accent action per view.
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);",
                Button { "Open the project" }
                Button {
                    variant: ButtonVariant::Accent,
                    icon_left: IconName::Plus,
                    on_click: move |_| drafts += 1,
                    "Draft summary"
                }
                Button { variant: ButtonVariant::Secondary, "Export sources" }
                Button { variant: ButtonVariant::Ghost, icon_right: IconName::ArrowRight, "View all" }
                Button { variant: ButtonVariant::Danger, icon_left: IconName::Trash, "Delete project" }
            }

            // Sizes: 32, 40 and 52px.
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);",
                Button { size: ButtonSize::Sm, "Book a demo" }
                Button { size: ButtonSize::Md, "Book a demo" }
                Button { size: ButtonSize::Lg, "Book a demo" }
            }

            // Disabled, and stretched to the container.
            Button { variant: ButtonVariant::Secondary, disabled: true, full_width: true, "Share with workspace" }

            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Drafts started: {drafts}" }
        }
    }
}
