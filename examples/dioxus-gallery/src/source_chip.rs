use dioxus::prelude::*;
use hydroxy_dioxus::icon::IconName;
use hydroxy_dioxus::source_chip::{SourceChip, SourceRow};

#[component]
pub fn SourceChipDemo() -> Element {
    let mut opened = use_signal(|| String::from("nothing yet"));
    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            p { style: "margin: 0; font: var(--text-body);", "Nine of twelve interviewees raised onboarding." }
            SourceRow {
                SourceChip { on_click: move |()| opened.set("interviews-q3.pdf".into()), "interviews-q3.pdf" }
                SourceChip { on_click: move |()| opened.set("tickets-sept.csv".into()), "tickets-sept.csv · p. 4" }
                SourceChip { icon: IconName::Inbox, "support inbox" }
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Opened: {opened}" }
        }
    }
}
