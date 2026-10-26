use dioxus::prelude::*;
use oxide_dioxus::status_diamond::{Jewel, StatusDiamond};

#[component]
pub fn StatusDiamondDemo() -> Element {
    let stages = [
        (Some(Jewel::Garnet), "Waiting"),
        (Some(Jewel::Citrine), "In review"),
        (Some(Jewel::Emerald), "Ready"),
        (Some(Jewel::Sapphire), "Shared"),
        (None, "Not started"),
    ];
    rsx! {
        ul { style: "list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);",
            for (jewel, name) in stages {
                li { style: "display: flex; align-items: center; gap: 10px; font: var(--text-sm);",
                    StatusDiamond { jewel }
                    "{name}"
                }
            }
            li { style: "display: flex; align-items: center; gap: 10px; font: var(--text-sm);",
                StatusDiamond { jewel: Jewel::Amethyst, hollow: true, size: 11, title: "Draft" }
                "Hollow, with an accessible name"
            }
        }
    }
}
