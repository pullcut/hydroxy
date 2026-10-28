use dioxus::prelude::*;
use hydroxy_dioxus::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};

#[component]
pub fn CollapsibleDemo() -> Element {
    let mut open = use_signal(|| Some(true));

    rsx! {
        // Controlled: the owner keeps the open state.
        Collapsible { open: open(), on_open_change: move |o| open.set(Some(o)),
            CollapsibleTrigger { "Sources" }
            CollapsibleContent {
                p { "interviews-q3.pdf" }
                p { "tickets-sept.csv" }
            }
        }

        // Uncontrolled, starts closed.
        Collapsible {
            CollapsibleTrigger { "Archived" }
            CollapsibleContent { "Nothing archived." }
        }

        // Disabled: the trigger ignores clicks.
        Collapsible { disabled: true,
            CollapsibleTrigger { "Shared with you" }
            CollapsibleContent { "No shared projects." }
        }
    }
}
