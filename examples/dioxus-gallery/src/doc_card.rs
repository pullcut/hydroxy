use dioxus::prelude::*;
use hydroxy_dioxus::avatar::Avatar;
use hydroxy_dioxus::doc_card::{DocCard, Jewel};
use hydroxy_dioxus::icon::IconName;

#[component]
pub fn DocCardDemo() -> Element {
    let mut pinned = use_signal(|| true);
    let mut opened = use_signal(|| String::from("nothing yet"));
    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: var(--space-5);",
                DocCard {
                    title: "Q3 customer research",
                    type_label: "Summary",
                    snippet: "Nine of twelve interviewees raised onboarding. Two items need your judgement before this is shared.",
                    pinned: pinned(),
                    on_toggle_pin: move |()| pinned.toggle(),
                    on_click: move |()| opened.set("Q3 customer research".into()),
                    footer: rsx! {
                        Avatar { initials: "JM", title: "Jo Marsh" }
                        span { style: "font: var(--text-xs); color: var(--text-3);", "Edited 26 October" }
                    },
                }
                DocCard {
                    title: "Support tickets, September",
                    type_label: "Spreadsheet",
                    icon: IconName::Inbox,
                    jewel: Jewel::Sapphire,
                    snippet: "410 fewer tickets than the August summary. The export may be missing a week.",
                    on_click: move |()| opened.set("Support tickets, September".into()),
                }
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Opened: {opened}" }
        }
    }
}
