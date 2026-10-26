use leptos::prelude::*;
use oxide_leptos::composer::Composer;

pub fn demo() -> AnyView {
    let (sent, set_sent) = signal(Vec::<String>::new());
    view! {
        <div style="display: grid; gap: var(--space-5); max-width: 560px;">
            <ul style="list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-3); font: var(--text-sm); color: var(--text-2);">
                {move || {
                    let messages = sent.get();
                    if messages.is_empty() {
                        view! { <li>"Nothing sent yet."</li> }.into_any()
                    } else {
                        messages.into_iter().map(|m| view! { <li>"Sent: " {m}</li> }).collect_view().into_any()
                    }
                }}
            </ul>
            <Composer
                suggestions=vec!["Summarise the interviews".to_string(), "What changed since August?".to_string()]
                on_send=move |m| set_sent.update(|s| s.push(m))
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "composer",
    name: "Composer",
    group: "Chat",
    summary: "The chat input. Enter sends, Shift+Enter adds a line, and the field grows with the message. Suggestions fill the field when picked.",
    demo,
    leptos: include_str!("composer.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/composer.rs"),
    parts: &[
        &["Composer", "—", "div[data-scope=composer]"],
        &["", "suggestions", "Tag buttons above the field"],
        &["", "field", "textarea plus the Send icon button"],
    ],
    keys: &[
        &["Enter", "Sends the message; blank messages are ignored"],
        &["Shift + Enter", "Adds a new line"],
    ],
    props: &[
        &["value", "Option<String>", "Controlled text. Pair with on_input"],
        &["on_input", "Callback<String>", "Called as the text changes"],
        &["on_send", "Callback<String>", "Called with the trimmed message; the field clears"],
        &["placeholder", "String", "Default \"Ask anything\""],
        &["label", "String", "Accessible name. Default \"Message\""],
        &["suggestions", "Vec<String>", "Tags shown above the field"],
        &["on_suggestion", "Callback<String>", "Picked suggestion. Without it, it fills the field"],
    ],
};
