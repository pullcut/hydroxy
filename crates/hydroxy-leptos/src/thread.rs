//! `<Thread>`: a conversation. A message scroller of chat bubbles over a composer, with a
//! typing indicator, a stop button while a reply streams, and copy and retry per message.

use std::time::Duration;

use hydroxy_headless::thread::{self as h, MessageAction, Role, ThreadMessage};
use leptos::either::Either;
use leptos::prelude::*;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::chat_bubble::{ChatBubble, ChatRole};
use crate::composer::Composer;
use crate::message_scroller::MessageScroller;

pub use hydroxy_headless::thread::{MessageAction as Action, Role as MessageRole, ThreadMessage as Message};

/// A chat thread. Controlled by the owner: pass `messages`, append in `on_send`.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn Thread(
    /// The messages, oldest first.
    #[prop(into)]
    messages: Signal<Vec<ThreadMessage>>,
    /// A reply is being written. Shows the typing indicator, then follows the text as it grows.
    #[prop(optional, into)]
    streaming: Signal<bool>,
    /// Called with the trimmed text when the person sends.
    #[prop(optional, into)]
    on_send: Option<Callback<String>>,
    /// Called when the person presses Stop. The button shows only while streaming.
    #[prop(optional, into)]
    on_stop: Option<Callback<()>>,
    /// Called with a message id and the action taken. Copy also writes to the clipboard.
    /// Retry shows only on a trailing assistant message, when this is set.
    #[prop(optional, into)]
    on_action: Option<Callback<(String, MessageAction)>>,
    /// Suggestion tags above the composer.
    #[prop(optional, into)]
    suggestions: Vec<String>,
    /// Accessible name for the message list.
    #[prop(default = "Conversation".into(), into)]
    label: String,
    /// Composer placeholder.
    #[prop(default = "Ask anything".into(), into)]
    placeholder: String,
    /// Text shown while there are no messages.
    #[prop(default = "No messages yet.".into(), into)]
    empty: String,
) -> impl IntoView {
    let copied = RwSignal::new(None::<String>);
    let count = Signal::derive(move || messages.with(Vec::len));
    let last_key = Signal::derive(move || messages.with(|m| m.last().map(|m| m.id.clone())));
    let revision = Signal::derive(move || messages.with(|m| m.last().map_or(0, |m| m.text.len())));
    let typing = move || messages.with(|m| h::typing_visible(m, streaming.get()));
    let retry = move || if streaming.get() { None } else { messages.with(|m| h::retry_target(m).map(str::to_string)) };

    let act = move |id: String, action: MessageAction| {
        if action == MessageAction::Copy
            && let Some(text) = messages.with(|m| m.iter().find(|m| m.id == id).map(|m| m.text.clone()))
        {
            // Best-effort: without clipboard access the button just doesn't confirm.
            let _ = window().navigator().clipboard().write_text(&text);
            copied.set(Some(id.clone()));
            set_timeout(
                move || {
                    copied.try_set(None);
                },
                Duration::from_millis(1400),
            );
        }
        if let Some(cb) = on_action {
            cb.run((id, action));
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-streaming=move || streaming.get().then_some("")>
            <div data-scope=h::SCOPE data-part="scroller">
                <MessageScroller count=count last_key=last_key revision=revision label=label>
                    {move || messages.with(Vec::is_empty).then(|| view! { <p data-scope=h::SCOPE data-part="empty">{empty.clone()}</p> })}
                    <For
                        each=move || messages.get()
                        key=|m| (m.id.clone(), m.text.clone())
                        children=move |m| {
                            let id = m.id.clone();
                            let role = if m.role == Role::User { ChatRole::User } else { ChatRole::Assistant };
                            let (copy_id, retry_id, copied_id) = (id.clone(), id.clone(), id.clone());
                            view! {
                                <div data-scope=h::SCOPE data-part="message" data-role=m.role.as_str()>
                                    {match m.time.clone() {
                                        Some(time) => {
                                            Either::Left(view! {
                                                <ChatBubble role=role sources=m.sources.clone() time=time>
                                                    <span data-scope=h::SCOPE data-part="text">{m.text.clone()}</span>
                                                </ChatBubble>
                                            })
                                        }
                                        None => {
                                            Either::Right(view! {
                                                <ChatBubble role=role sources=m.sources.clone()>
                                                    <span data-scope=h::SCOPE data-part="text">{m.text.clone()}</span>
                                                </ChatBubble>
                                            })
                                        }
                                    }}
                                    <div data-scope=h::SCOPE data-part="actions">
                                        <button
                                            type="button"
                                            data-scope=h::SCOPE
                                            data-part="action"
                                            on:click=move |_| act(copy_id.clone(), MessageAction::Copy)
                                        >
                                            {move || if copied.get().as_deref() == Some(&copied_id) { "Copied" } else { "Copy" }}
                                        </button>
                                        {move || {
                                            let retry_id = retry_id.clone();
                                            (on_action.is_some() && retry().as_deref() == Some(&retry_id))
                                                .then(|| {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            data-scope=h::SCOPE
                                                            data-part="action"
                                                            on:click=move |_| act(retry_id.clone(), MessageAction::Retry)
                                                        >
                                                            "Retry"
                                                        </button>
                                                    }
                                                })
                                        }}
                                    </div>
                                </div>
                            }
                        }
                    />
                    {move || {
                        typing()
                            .then(|| {
                                view! {
                                    <div data-scope=h::SCOPE data-part="typing" role="status" aria-label="Assistant is typing">
                                        <span></span>
                                        <span></span>
                                        <span></span>
                                    </div>
                                }
                            })
                    }}
                </MessageScroller>
            </div>
            <div data-scope=h::SCOPE data-part="composer">
                {move || {
                    (streaming.get() && on_stop.is_some())
                        .then(|| {
                            view! {
                                <div data-scope=h::SCOPE data-part="stop">
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        size=ButtonSize::Sm
                                        on_click=move |_| {
                                            if let Some(cb) = on_stop {
                                                cb.run(());
                                            }
                                        }
                                    >
                                        "Stop generating"
                                    </Button>
                                </div>
                            }
                        })
                }}
                <Composer on_send=move |t: String| {
                    if let Some(cb) = on_send {
                        cb.run(t);
                    }
                } placeholder=placeholder suggestions=suggestions />
            </div>
        </div>
    }
}
