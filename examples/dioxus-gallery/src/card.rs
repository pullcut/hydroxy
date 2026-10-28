use dioxus::prelude::*;
use hydroxy_dioxus::card::{Card, CardTone, Jewel};

#[component]
pub fn CardDemo() -> Element {
    let mut opened = use_signal(|| 0);

    rsx! {
        div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: var(--space-5);",
            Card {
                p { style: "margin: 0 0 4px; font: var(--text-label); letter-spacing: var(--tracking-label); text-transform: uppercase; color: var(--text-3);", "Project" }
                p { style: "margin: 0; font: var(--text-h4);", "Q3 customer research" }
                p { style: "margin: 8px 0 0; font: var(--text-sm); color: var(--text-2);", "12 files · 3 summaries" }
            }
            Card { on_click: move |()| opened += 1,
                p { style: "margin: 0; font: var(--text-h4);", "Onboarding interviews" }
                p { style: "margin: 8px 0 0; font: var(--text-sm); color: var(--text-2);", "Opened {opened} times" }
            }
            Card { tone: CardTone::Sunken,
                p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Drafts stay here until you share them." }
            }
            Card { tone: CardTone::Inverse,
                p { style: "margin: 0; font: var(--text-figure);", "22 of 24" }
                p { style: "margin: 8px 0 0; font: var(--text-sm);", "Interviews read" }
            }
            Card { tone: CardTone::Jewel(Jewel::Amethyst),
                p { style: "margin: 0; font: var(--text-h4);", "Read" }
                p { style: "margin: 8px 0 0; font: var(--text-sm);", "Every file, with the page for each figure." }
            }
            Card { tone: CardTone::Jewel(Jewel::Citrine),
                p { style: "margin: 0; font: var(--text-h4);", "Flag" }
                p { style: "margin: 8px 0 0; font: var(--text-sm);", "Two items need your judgement." }
            }
        }
    }
}
