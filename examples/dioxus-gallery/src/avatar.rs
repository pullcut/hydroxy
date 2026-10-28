use dioxus::prelude::*;
use hydroxy_dioxus::avatar::{Avatar, Jewel};

#[component]
pub fn AvatarDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            // People, an agent and an unassigned slot at the default 22px.
            div { style: "display: flex; align-items: center; gap: var(--space-3);",
                Avatar { initials: "JM", jewel: Jewel::Sapphire, title: "Jo Marsh" }
                Avatar { initials: "AK", jewel: Jewel::Emerald, title: "Ana Kovac" }
                Avatar { initials: "RT", jewel: Jewel::Garnet, title: "Rui Tavares" }
                Avatar { initials: "hy", agent: true, title: "hydroxy assistant" }
                Avatar {}
            }

            // Larger sizes.
            div { style: "display: flex; align-items: center; gap: var(--space-3);",
                Avatar { initials: "JM", jewel: Jewel::Sapphire, size: 32, title: "Jo Marsh" }
                Avatar { initials: "hy", agent: true, size: 32, title: "hydroxy assistant" }
                Avatar { initials: "AK", jewel: Jewel::Emerald, size: 40, title: "Ana Kovac" }
                Avatar { size: 40 }
            }
        }
    }
}
