use leptos::prelude::*;
use oxide_leptos::chat_bubble::{ChatBubble, ChatRole};

pub fn demo() -> AnyView {
    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 560px;">
            <ChatBubble role=ChatRole::User time="09:41">"What did people say about onboarding?"</ChatBubble>
            <ChatBubble sources=vec!["interviews-q3.pdf".to_string(), "survey-results.xlsx".to_string()] time="09:41">
                "Nine of twelve interviewees raised onboarding. Most wanted a shorter first setup; three asked for sample data."
            </ChatBubble>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "chat-bubble",
    name: "Chat bubble",
    group: "Chat",
    summary: "A message in an assistant chat. The assistant's answers sit on the page with the mark and their sources; the person's messages are inverse bubbles on the right.",
    demo,
    leptos: include_str!("chat_bubble.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/chat_bubble.rs"),
    parts: &[
        &["ChatBubble", "—", "div[data-role=assistant|user]"],
        &["", "avatar", "The mark, assistant messages only"],
        &["", "message", "The text; an inverse bubble for the person"],
        &["", "time", "Optional time"],
    ],
    keys: &[],
    props: &[
        &["role", "ChatRole", "Assistant (default) or User"],
        &["sources", "Vec<String>", "File names behind an answer"],
        &["time", "String", "When it was sent"],
        &["children", "View", "The message"],
    ],
};
