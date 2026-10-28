use dioxus::prelude::*;
use hydroxy_dioxus::icon::{Icon, IconName};
use hydroxy_dioxus::toggle_button::ToggleButton;

#[component]
pub fn ToggleButtonDemo() -> Element {
    let mut starred = use_signal(|| false);

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);",
            ToggleButton {
                pressed: Some(starred()),
                on_pressed_change: move |v| starred.set(v),
                Icon { name: IconName::Star, size: 14 }
                if starred() { "Starred" } else { "Star" }
            }
            ToggleButton { label: "Pin", default_pressed: true,
                Icon { name: IconName::Mark, size: 14 }
            }
            ToggleButton { "Show sources" }
            ToggleButton { disabled: true, "Disabled" }
        }
    }
}
