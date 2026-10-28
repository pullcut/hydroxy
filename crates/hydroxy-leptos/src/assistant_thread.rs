//! `<AssistantThread>` and the pieces of one: messages and their parts, the action bar,
//! branch picker, chain of thought, error and the quote-selection toolbar.
//!
//! The owner holds the messages. The thread renders them in a [`MessageScroller`], and
//! leaves a `children` slot below for a composer.

use std::collections::HashMap;

use hydroxy_headless::assistant_thread as h;
use leptos::prelude::*;

use crate::message_scroller::MessageScroller;

mod action_bar;
mod branch_picker;
mod message;
mod selection_toolbar;

pub use action_bar::*;
pub use branch_picker::*;
pub use hydroxy_headless::assistant_thread::{
    ActionBarState, AssistantMessageData, Autohide, Feedback, Part, PartStatus, Role, Selection,
};
pub use message::*;
pub use selection_toolbar::*;

// ------------------------------------------------------------------- thread

/// A scrolling assistant conversation. Pass the messages and a `running` flag; apply
/// the callbacks to your own data. Children render below the messages (put the composer
/// there).
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::implicit_hasher,
    reason = "one view! tree and a std HashMap prop; splitting or generalising only adds plumbing"
)]
pub fn AssistantThread(
    /// The messages, oldest first.
    #[prop(into)]
    messages: Signal<Vec<AssistantMessageData>>,
    /// A reply is running: shows the typing indicator and disables Copy and branch switching.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// Branch position per message id: `(zero-based index, count)`. Messages with more than one branch show the picker.
    #[prop(optional, into)]
    branches: Signal<HashMap<String, (usize, usize)>>,
    /// Hover-reveal rule for the action bars.
    #[prop(optional)]
    autohide: Autohide,
    /// Hide the action bars while a reply runs.
    #[prop(optional)]
    hide_when_running: bool,
    /// Called with the message id when Reload is pressed.
    #[prop(optional, into)]
    on_reload: Option<Callback<String>>,
    /// Called with the message id when Edit is pressed.
    #[prop(optional, into)]
    on_edit: Option<Callback<String>>,
    /// Called with the message id and the vote (`None` when taken back).
    #[prop(optional, into)]
    on_feedback: Option<Callback<(String, Option<Feedback>)>>,
    /// Called with the message id and its Markdown when Export is pressed.
    #[prop(optional, into)]
    on_export: Option<Callback<(String, String)>>,
    /// Called with the message id and the branch to show.
    #[prop(optional, into)]
    on_branch_switch: Option<Callback<(String, usize)>>,
    /// Called with the message id and whether reading aloud should start.
    #[prop(optional, into)]
    on_speak: Option<Callback<(String, bool)>>,
    /// Called with the message id when Try again is pressed on an error.
    #[prop(optional, into)]
    on_retry: Option<Callback<String>>,
    /// When set, selecting text in a message shows a Quote button that calls this.
    #[prop(optional, into)]
    on_quote: Option<Callback<Selection>>,
    /// Accessible name for the message list.
    #[prop(default = "Conversation".into(), into)]
    label: String,
    /// Text shown while there are no messages.
    #[prop(default = "No messages yet.".into(), into)]
    empty: String,
    /// Below the messages, such as a composer.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let count = Signal::derive(move || messages.with(Vec::len));
    let last_key = Signal::derive(move || messages.with(|m| m.last().map(|m| m.id.clone())));
    let revision = Signal::derive(move || {
        messages.with(|m| {
            m.last().map_or(0, |m| {
                m.parts.iter().map(|p| format!("{p:?}").len()).sum::<usize>() + m.error.as_ref().map_or(0, String::len)
            })
        })
    });
    let ids = Signal::derive(move || messages.with(|m| m.iter().map(|m| m.id.clone()).collect::<Vec<_>>()));
    let typing =
        move || running.get() && messages.with(|m| m.last().is_none_or(|m| m.role == Role::User || m.parts.is_empty()));

    view! {
        <div data-scope=h::SCOPE_THREAD data-part="root" data-running=move || running.get().then_some("")>
            <div data-scope=h::SCOPE_THREAD data-part="scroller">
                <MessageScroller count=count last_key=last_key revision=revision label=label>
                    {move || {
                        messages
                            .with(Vec::is_empty)
                            .then(|| view! { <p data-scope=h::SCOPE_THREAD data-part="empty">{empty.clone()}</p> })
                    }}
                    <For
                        each=move || ids.get()
                        key=Clone::clone
                        children=move |id| {
                            let message = Memo::new({
                                let id = id.clone();
                                move |_| {
                                    messages
                                        .with(|v| v.iter().find(|m| m.id == id).cloned())
                                        .unwrap_or_else(|| AssistantMessageData::text(id.clone(), Role::Assistant, ""))
                                }
                            });
                            let is_last = Signal::derive({
                                let id = id.clone();
                                move || messages.with(|v| v.last().is_some_and(|m| m.id == id))
                            });
                            let branch = Signal::derive({
                                let id = id.clone();
                                move || branches.with(|b| b.get(&id).copied())
                            });
                            let with_id = |cb: Option<Callback<String>>, id: String| {
                                Callback::new(move |(): ()| {
                                    if let Some(cb) = cb {
                                        cb.run(id.clone());
                                    }
                                })
                            };
                            view! {
                                <AssistantMessage
                                    message=message
                                    running=running
                                    is_last=is_last
                                    branch=branch
                                    autohide=autohide
                                    hide_when_running=hide_when_running
                                    on_reload=with_id(on_reload, id.clone())
                                    on_edit=with_id(on_edit, id.clone())
                                    on_retry=with_id(on_retry, id.clone())
                                    on_feedback={
                                        let id = id.clone();
                                        Callback::new(move |v| {
                                            if let Some(cb) = on_feedback {
                                                cb.run((id.clone(), v));
                                            }
                                        })
                                    }
                                    on_export={
                                        let id = id.clone();
                                        Callback::new(move |md| {
                                            if let Some(cb) = on_export {
                                                cb.run((id.clone(), md));
                                            }
                                        })
                                    }
                                    on_branch_switch={
                                        let id = id.clone();
                                        Callback::new(move |i| {
                                            if let Some(cb) = on_branch_switch {
                                                cb.run((id.clone(), i));
                                            }
                                        })
                                    }
                                    on_speak={
                                        let id = id.clone();
                                        Callback::new(move |on| {
                                            if let Some(cb) = on_speak {
                                                cb.run((id.clone(), on));
                                            }
                                        })
                                    }
                                />
                            }
                        }
                    />
                    {move || {
                        typing()
                            .then(|| {
                                view! {
                                    <div data-scope=h::SCOPE_THREAD data-part="typing" role="status" aria-label="Assistant is typing">
                                        <span></span>
                                        <span></span>
                                        <span></span>
                                    </div>
                                }
                            })
                    }}
                </MessageScroller>
            </div>
            {children.map(|c| view! { <div data-scope=h::SCOPE_THREAD data-part="footer">{c()}</div> })}
            {on_quote.map(|cb| view! { <AssistantSelectionToolbar on_quote=cb /> })}
        </div>
    }
}
