//! `<ChatBubble>`: a message in an assistant chat. Assistant messages sit on the page with
//! the mark; user messages are inverse bubbles on the right.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};
use crate::source_chip::{SourceChip, SourceRow};

/// Who wrote the message.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ChatRole {
    /// The assistant: on the page, with the mark.
    #[default]
    Assistant,
    /// The person: an inverse bubble on the right.
    User,
}

/// One chat message, with optional sources and time.
#[component]
pub fn ChatBubble(
    /// Who wrote it. Default the assistant.
    #[prop(optional)]
    role: ChatRole,
    /// File names behind an answer, shown as source chips.
    #[prop(optional, into)]
    sources: Vec<String>,
    /// When it was sent, such as "09:42".
    #[prop(optional, into)]
    time: Option<String>,
    /// The message.
    children: Children,
) -> impl IntoView {
    let user = role == ChatRole::User;
    view! {
        <div data-scope="chat-bubble" data-role=if user { "user" } else { "assistant" }>
            {(!user).then(|| view! { <span data-part="avatar"><Icon name=IconName::Mark size=28 /></span> })}
            <div data-part="stack">
                <div data-part="message">{children()}</div>
                {(!sources.is_empty())
                    .then(|| {
                        view! {
                            <SourceRow>
                                {sources.into_iter().map(|s| view! { <SourceChip>{s}</SourceChip> }).collect_view()}
                            </SourceRow>
                        }
                    })}
                {time.map(|t| view! { <span data-part="time">{t}</span> })}
            </div>
        </div>
    }
}
