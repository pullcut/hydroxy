use dioxus::prelude::*;
use oxide_dioxus::button::ButtonVariant;
use oxide_dioxus::popover::{Align, Placement, Popover, PopoverClose, PopoverContent, PopoverTrigger, Side};

#[component]
pub fn PopoverDemo() -> Element {
    let mut open = use_signal(|| Some(false));

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-5);",
            // Uncontrolled. Opens below the trigger and focuses the first button inside.
            Popover {
                PopoverTrigger { "Sharing" }
                PopoverContent {
                    p { style: "margin: 0 0 4px; font: var(--text-ui);", "Shared with the workspace" }
                    p { style: "margin: 0 0 12px; color: var(--text-2);",
                        "22 of 24 people have opened it. Sources stay attached."
                    }
                    PopoverClose { variant: ButtonVariant::Secondary, "Done" }
                }
            }

            // Controlled, to the side. Escape closes and returns focus to the trigger.
            Popover {
                open: open(),
                on_open_change: move |v| open.set(Some(v)),
                placement: Placement::new(Side::End, Align::Start),
                PopoverTrigger { variant: ButtonVariant::Ghost, "Where this figure comes from" }
                PopoverContent {
                    p { style: "margin: 0; color: var(--text-2);",
                        "Nine of twelve interviewees raised onboarding. From interviews-q3.pdf, pages 4 to 11."
                    }
                }
            }
            span { "Open: {open().unwrap_or(false)}" }
        }
    }
}
