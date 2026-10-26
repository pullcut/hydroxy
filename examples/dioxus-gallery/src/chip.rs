use dioxus::prelude::*;
use oxide_dioxus::chip::{Chip, Jewel};

#[component]
pub fn ChipDemo() -> Element {
    let mut labels = use_signal(|| vec!["onboarding", "billing", "export"]);

    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-2);",
                Chip { jewel: Jewel::Garnet, "P0" }
                Chip { jewel: Jewel::Sapphire, "OX-142" }
                Chip { "22 of 24" }
                Chip { mono: false, "Design review" }
            }

            // Removable.
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-2);",
                for label in labels() {
                    Chip {
                        key: "{label}",
                        jewel: Jewel::Amethyst,
                        on_remove: move |()| labels.write().retain(|l| *l != label),
                        "{label}"
                    }
                }
            }
        }
    }
}
