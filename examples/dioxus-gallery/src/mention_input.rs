use dioxus::prelude::*;
use hydroxy_dioxus::mention_input::{Mention, MentionInput, MentionItem, MentionSource, MentionValue};

fn sources() -> Vec<MentionSource> {
    let people = vec![
        MentionItem::new("u_ana", "Ana Lima").description("Design lead"),
        MentionItem::new("u_bo", "Bo Chen").description("Backend"),
        MentionItem::new("u_dana", "Dana Ortiz").description("Product"),
        MentionItem::new("u_joan", "Joan Park").description("Support"),
        MentionItem::new("u_kofi", "Kofi Mensah").description("Data"),
        MentionItem::new("u_mira", "Mira Shah").description("Frontend"),
        MentionItem::new("u_sam", "Sam Okafor").description("QA"),
    ];
    let commands = vec![
        MentionItem::new("assign", "assign").description("Assign to someone"),
        MentionItem::new("due", "due").description("Set a due date"),
        MentionItem::new("label", "label").description("Add a label"),
        MentionItem::new("estimate", "estimate").description("Set points"),
        MentionItem::new("close", "close").description("Close the issue"),
    ];
    vec![MentionSource::new('@', "People", people), MentionSource::new('/', "Commands", commands)]
}

#[component]
pub fn MentionInputDemo() -> Element {
    let start = "Checkout fails on Safari for some cards. @Bo Chen can you look at the logs? ";
    let initial = MentionValue {
        text: start.into(),
        mentions: vec![Mention { id: "u_bo".into(), label: "Bo Chen".into(), trigger: '@', start: 41, end: 49 }],
    };
    let mut value = use_signal(|| initial.clone());
    let v = value.read();
    let count = v.text.chars().count();
    let mentions = v.mentions.clone();
    drop(v);

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 560px;",
            MentionInput {
                label: "Comment",
                sources: sources(),
                default_value: initial,
                placeholder: "Write a comment. @ mentions someone, / runs a command",
                on_change: move |v: MentionValue| value.set(v),
            }
            div { style: "display: grid; gap: var(--space-1); font: var(--text-xs); color: var(--text-3);",
                span { "{count} characters" }
                for m in mentions {
                    code { style: "font-family: var(--font-mono);",
                        "{m.trigger}{m.label} → {m.id} at {m.start}..{m.end}"
                    }
                }
            }
        }
    }
}
