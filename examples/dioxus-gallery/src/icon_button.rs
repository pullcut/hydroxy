use dioxus::prelude::*;
use hydroxy_dioxus::button::{ButtonSize, ButtonVariant};
use hydroxy_dioxus::icon::IconName;
use hydroxy_dioxus::icon_button::IconButton;

#[component]
pub fn IconButtonDemo() -> Element {
    let mut copied = use_signal(|| false);

    rsx! {
        div { style: "display: grid; gap: var(--space-7);",
            // Variants. Ghost is the default for toolbars.
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);",
                IconButton { icon: IconName::Copy, label: "Copy link", on_click: move |_| copied.set(true) }
                IconButton { icon: IconName::Ellipsis, label: "More actions", variant: ButtonVariant::Secondary }
                IconButton { icon: IconName::Plus, label: "New project", variant: ButtonVariant::Primary }
                IconButton { icon: IconName::Search, label: "Search sources", variant: ButtonVariant::Accent }
                IconButton { icon: IconName::Trash, label: "Delete project", disabled: true }
            }

            // Sizes: 32, 40 and 48px squares.
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);",
                IconButton { icon: IconName::X, label: "Close", variant: ButtonVariant::Secondary, size: ButtonSize::Sm }
                IconButton { icon: IconName::X, label: "Close", variant: ButtonVariant::Secondary, size: ButtonSize::Md }
                IconButton { icon: IconName::X, label: "Close", variant: ButtonVariant::Secondary, size: ButtonSize::Lg }
            }

            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                if copied() { "Link copied." } else { "Hover a button to see its label." }
            }
        }
    }
}
