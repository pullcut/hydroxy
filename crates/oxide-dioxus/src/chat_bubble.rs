//! `ChatBubble`: a message in an assistant chat. Assistant messages sit on the page with
//! the mark; user messages are inverse bubbles on the right.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};
use crate::source_chip::{SourceChip, SourceRow};

/// Who wrote the message.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
    #[props(default)]
    role: ChatRole,
    /// File names behind an answer, shown as source chips.
    #[props(default)]
    sources: Vec<String>,
    /// When it was sent, such as "09:42".
    time: Option<String>,
    /// The message.
    children: Element,
) -> Element {
    let user = role == ChatRole::User;
    rsx! {
        div { "data-scope": "chat-bubble", "data-role": if user { "user" } else { "assistant" },
            if !user {
                span { "data-part": "avatar",
                    Icon { name: IconName::Mark, size: 28 }
                }
            }
            div { "data-part": "stack",
                div { "data-part": "message", {children} }
                if !sources.is_empty() {
                    SourceRow {
                        for s in sources {
                            SourceChip { key: "{s}", "{s}" }
                        }
                    }
                }
                if let Some(t) = time {
                    span { "data-part": "time", "{t}" }
                }
            }
        }
    }
}
