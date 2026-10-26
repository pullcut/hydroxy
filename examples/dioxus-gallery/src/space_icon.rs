use dioxus::prelude::*;
use oxide_dioxus::icon::IconName;
use oxide_dioxus::space_icon::{Jewel, SpaceIcon};

#[component]
pub fn SpaceIconDemo() -> Element {
    let spaces = [
        (IconName::FileText, Jewel::Amethyst, "Research"),
        (IconName::Inbox, Jewel::Sapphire, "Support"),
        (IconName::Lightbulb, Jewel::Citrine, "Product ideas"),
        (IconName::User, Jewel::Emerald, "People"),
    ];
    rsx! {
        div { style: "display: grid; gap: var(--space-7);",
            ul { style: "list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);",
                for (icon, jewel, name) in spaces {
                    li { style: "display: flex; align-items: center; gap: 10px; font: var(--text-ui);",
                        SpaceIcon { icon, jewel }
                        "{name}"
                    }
                }
            }
            div { style: "display: flex; align-items: center; gap: var(--space-4);",
                SpaceIcon { icon: IconName::FileText, size: 20 }
                SpaceIcon { icon: IconName::FileText, size: 32 }
                SpaceIcon { icon: IconName::FileText, size: 48, title: "Research team" }
            }
        }
    }
}
