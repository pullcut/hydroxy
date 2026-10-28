use dioxus::prelude::*;
use hydroxy_dioxus::reveal::{Reveal, RevealReason};

fn card(title: &'static str, body: &'static str) -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-2); padding: var(--space-6); border: 1px solid var(--border-1); border-radius: var(--radius-md); background: var(--surface-card);",
            strong { style: "font: var(--text-ui);", {title} }
            span { style: "font: var(--text-sm); color: var(--text-3);", {body} }
        }
    }
}

#[component]
pub fn RevealDemo() -> Element {
    let mut why = use_signal(|| String::from("Scroll down. Each card fades in as it arrives."));

    rsx! {
        div { style: "display: grid; gap: 38vh;",
            p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", "{why}" }
            Reveal { on_reveal: move |r: RevealReason| why.set(format!("Last revealed: {r:?}.")),
                {card("Import", "Bring in notes, tickets and survey exports.")}
            }
            Reveal { on_reveal: move |r: RevealReason| why.set(format!("Last revealed: {r:?}.")),
                {card("Tag", "Group what you find into themes.")}
            }
            Reveal { on_reveal: move |r: RevealReason| why.set(format!("Last revealed: {r:?}.")),
                {card("Share", "Send a digest to the team.")}
            }
        }
    }
}
