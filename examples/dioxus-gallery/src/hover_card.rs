use dioxus::prelude::*;
use hydroxy_dioxus::hover_card::{HoverCard, HoverCardContent, HoverCardTrigger};

#[component]
pub fn HoverCardDemo() -> Element {
    rsx! {
        // A div, not a p: the card is a block element.
        div {
            "Summary drafted by "
            HoverCard {
                HoverCardTrigger { href: "#", "@maya" }
                HoverCardContent {
                    div { style: "display: grid; gap: var(--space-2);",
                        strong { "Maya Okafor" }
                        span { style: "color: var(--text-2);",
                            "Research lead. Owns the Q3 customer interviews and 4 summaries."
                        }
                    }
                }
            }
            " from 18 interviews."
        }
    }
}
