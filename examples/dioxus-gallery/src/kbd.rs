use dioxus::prelude::*;
use hydroxy_dioxus::kbd::Kbd;

#[component]
pub fn KbdDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-4); font: var(--text-sm); color: var(--text-2);",
            p { style: "margin: 0; display: flex; align-items: center; gap: var(--space-2);",
                "Search everything"
                Kbd { "⌘" }
                Kbd { "K" }
            }
            p { style: "margin: 0; display: flex; align-items: center; gap: var(--space-2);",
                "New line"
                Kbd { "Shift" }
                Kbd { "Enter" }
            }
            p { style: "margin: 0; display: flex; align-items: center; gap: var(--space-2);",
                "Close the panel"
                Kbd { "Esc" }
            }
        }
    }
}
