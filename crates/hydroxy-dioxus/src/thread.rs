//! `Thread`: a conversation. A message scroller of chat bubbles over a composer, with a
//! typing indicator, a stop button while a reply streams, and copy and retry per message.

use dioxus::prelude::*;
use hydroxy_headless::thread::{self as h, MessageAction, Role, ThreadMessage};

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::chat_bubble::{ChatBubble, ChatRole};
use crate::composer::Composer;
use crate::js;
use crate::message_scroller::MessageScroller;

pub use hydroxy_headless::thread::{MessageAction as Action, Role as MessageRole, ThreadMessage as Message};

/// A chat thread. Controlled by the owner: pass `messages`, append in `on_send`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn Thread(
    /// The messages, oldest first.
    messages: ReadSignal<Vec<ThreadMessage>>,
    /// A reply is being written. Shows the typing indicator, then follows the text as it grows.
    #[props(default)]
    streaming: ReadSignal<bool>,
    /// Called with the trimmed text when the person sends.
    on_send: Option<EventHandler<String>>,
    /// Called when the person presses Stop. The button shows only while streaming.
    on_stop: Option<EventHandler<()>>,
    /// Called with a message id and the action taken. Copy also writes to the clipboard.
    /// Retry shows only on a trailing assistant message, when this is set.
    on_action: Option<EventHandler<(String, MessageAction)>>,
    /// Suggestion tags above the composer.
    #[props(default)]
    suggestions: Vec<String>,
    /// Accessible name for the message list.
    #[props(default = "Conversation".to_string())]
    label: String,
    /// Text shown while there are no messages.
    #[props(default = "No messages yet.".to_string())]
    empty: String,
) -> Element {
    let mut copied = use_signal(|| None::<String>);
    let msgs = messages.read();
    let last_key = msgs.last().map(|m| m.id.clone());
    let revision = msgs.last().map_or(0, |m| m.text.len());
    let typing = h::typing_visible(&msgs, streaming());
    let retry = if streaming() { None } else { h::retry_target(&msgs).map(str::to_string) };

    let act = move |id: String, action: MessageAction| {
        if action == MessageAction::Copy {
            let text = messages.peek().iter().find(|m| m.id == id).map(|m| m.text.clone());
            if let Some(text) = text {
                let copy_id = id.clone();
                spawn(async move {
                    js::copy(&text);
                    copied.set(Some(copy_id.clone()));
                    js::sleep(1400).await;
                    if copied.peek().as_deref() == Some(&copy_id) {
                        copied.set(None);
                    }
                });
            }
        }
        if let Some(handler) = on_action {
            handler.call((id, action));
        }
    };

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-streaming": streaming().then_some(""),
            div { "data-scope": h::SCOPE, "data-part": "scroller",
                MessageScroller { count: msgs.len(), last_key, revision, label,
                    if msgs.is_empty() {
                        p { "data-scope": h::SCOPE, "data-part": "empty", "{empty}" }
                    }
                    for m in msgs.iter() {
                        {
                            let role = if m.role == Role::User { ChatRole::User } else { ChatRole::Assistant };
                            let (copy_id, retry_id) = (m.id.clone(), m.id.clone());
                            let is_copied = copied.read().as_deref() == Some(&m.id);
                            let can_retry = on_action.is_some() && retry.as_deref() == Some(&m.id);
                            rsx! {
                                div {
                                    key: "{m.id}",
                                    "data-scope": h::SCOPE,
                                    "data-part": "message",
                                    "data-role": m.role.as_str(),
                                    ChatBubble { role, sources: m.sources.clone(), time: m.time.clone(),
                                        span { "data-scope": h::SCOPE, "data-part": "text", "{m.text}" }
                                    }
                                    div { "data-scope": h::SCOPE, "data-part": "actions",
                                        button {
                                            r#type: "button",
                                            "data-scope": h::SCOPE,
                                            "data-part": "action",
                                            onclick: move |_| act(copy_id.clone(), MessageAction::Copy),
                                            if is_copied {
                                                "Copied"
                                            } else {
                                                "Copy"
                                            }
                                        }
                                        if can_retry {
                                            button {
                                                r#type: "button",
                                                "data-scope": h::SCOPE,
                                                "data-part": "action",
                                                onclick: move |_| act(retry_id.clone(), MessageAction::Retry),
                                                "Retry"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if typing {
                        div {
                            "data-scope": h::SCOPE,
                            "data-part": "typing",
                            role: "status",
                            "aria-label": "Assistant is typing",
                            span {}
                            span {}
                            span {}
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "composer",
                if streaming() && on_stop.is_some() {
                    div { "data-scope": h::SCOPE, "data-part": "stop",
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            on_click: move |_| {
                                if let Some(handler) = on_stop {
                                    handler.call(());
                                }
                            },
                            "Stop generating"
                        }
                    }
                }
                Composer {
                    suggestions,
                    on_send: move |t: String| {
                        if let Some(handler) = on_send {
                            handler.call(t);
                        }
                    },
                }
            }
        }
    }
}
