use dioxus::prelude::*;
use hydroxy_dioxus::tag::{Jewel, Tag};

#[component]
pub fn TagDemo() -> Element {
    let mut interviews = use_signal(|| true);
    let mut tickets = use_signal(|| false);
    let mut sources = use_signal(|| vec!["interviews-q3.pdf", "tickets-sept.csv", "nps-august.xlsx"]);

    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            // Toggle filters: a clickable tag is a button with aria-pressed.
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
                Tag {
                    jewel: Jewel::Amethyst,
                    selected: interviews(),
                    on_click: move |()| interviews.toggle(),
                    "Interviews"
                }
                Tag {
                    jewel: Jewel::Sapphire,
                    selected: tickets(),
                    on_click: move |()| tickets.toggle(),
                    "Support tickets"
                }
                Tag { jewel: Jewel::Citrine, "Q3 research" }
            }

            // Removable.
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
                for source in sources() {
                    Tag {
                        key: "{source}",
                        jewel: Jewel::Emerald,
                        on_remove: move |()| sources.write().retain(|s| *s != source),
                        "{source}"
                    }
                }
            }
        }
    }
}
