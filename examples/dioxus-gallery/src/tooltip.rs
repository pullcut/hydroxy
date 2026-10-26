use dioxus::prelude::*;
use oxide_dioxus::tooltip::{Align, Placement, Side, Tooltip, TooltipContent, TooltipTrigger};

#[component]
pub fn TooltipDemo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-4);",
            // Default: above the trigger, centred.
            Tooltip {
                TooltipTrigger { "Copy link" }
                TooltipContent { "Copies a link to this summary" }
            }
            Tooltip { placement: Placement::new(Side::Bottom, Align::Start),
                TooltipTrigger { "Sources" }
                TooltipContent { "3 files, read 26 October" }
            }
            Tooltip { placement: Placement::new(Side::End, Align::Center),
                TooltipTrigger { "Retry" }
                TooltipContent { "Runs the checks again" }
            }
            Tooltip { disabled: true,
                TooltipTrigger { "No tooltip" }
                TooltipContent { "Never shown" }
            }
        }
    }
}
