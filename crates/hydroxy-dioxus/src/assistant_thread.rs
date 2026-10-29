//! `AssistantThread` and the pieces of one: messages and their parts, the action bar,
//! branch picker, chain of thought, error and the quote-selection toolbar.
//!
//! The owner holds the messages. The thread renders them in a `MessageScroller`, and
//! leaves a `children` slot below for a composer.

mod action_bar;
mod branch_picker;
mod message;
mod selection_toolbar;

use std::collections::HashMap;

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread as h;

use crate::message_scroller::MessageScroller;

pub use action_bar::AssistantActionBar;
pub use branch_picker::AssistantBranchPicker;
pub use hydroxy_headless::assistant_thread::{
    ActionBarState, AssistantMessageData, Autohide, Feedback, Part, PartStatus, Role, Selection,
};
pub use message::{AssistantChainOfThought, AssistantError, AssistantMessage};
pub use selection_toolbar::AssistantSelectionToolbar;

// ------------------------------------------------------------------- thread

/// A scrolling assistant conversation. Pass the messages and a `running` flag; apply
/// the callbacks to your own data. Children render below the messages (put the composer
/// there).
#[component]
pub fn AssistantThread(
    /// The messages, oldest first.
    messages: ReadSignal<Vec<AssistantMessageData>>,
    /// A reply is running: shows the typing indicator and disables Copy and branch switching.
    #[props(default)]
    running: ReadSignal<bool>,
    /// Branch position per message id: `(zero-based index, count)`. Messages with more than one branch show the picker.
    #[props(default)]
    branches: ReadSignal<HashMap<String, (usize, usize)>>,
    /// Hover-reveal rule for the action bars.
    #[props(default)]
    autohide: Autohide,
    /// Hide the action bars while a reply runs.
    #[props(default)]
    hide_when_running: bool,
    /// Called with the message id when Reload is pressed.
    on_reload: Option<EventHandler<String>>,
    /// Called with the message id when Edit is pressed.
    on_edit: Option<EventHandler<String>>,
    /// Called with the message id and the vote (`None` when taken back).
    on_feedback: Option<EventHandler<(String, Option<Feedback>)>>,
    /// Called with the message id and its Markdown when Export is pressed.
    on_export: Option<EventHandler<(String, String)>>,
    /// Called with the message id and the branch to show.
    on_branch_switch: Option<EventHandler<(String, usize)>>,
    /// Called with the message id and whether reading aloud should start.
    on_speak: Option<EventHandler<(String, bool)>>,
    /// Called with the message id when Try again is pressed on an error.
    on_retry: Option<EventHandler<String>>,
    /// When set, selecting text in a message shows a Quote button that calls this.
    on_quote: Option<EventHandler<Selection>>,
    /// Accessible name for the message list.
    #[props(default = "Conversation".to_string())]
    label: String,
    /// Text shown while there are no messages.
    #[props(default = "No messages yet.".to_string())]
    empty: String,
    /// Below the messages, such as a composer.
    children: Element,
) -> Element {
    let msgs = messages.read();
    let branch_map = branches.read();
    let last_key = msgs.last().map(|m| m.id.clone());
    let revision = msgs.last().map_or(0, |m| {
        m.parts.iter().map(|p| format!("{p:?}").len()).sum::<usize>() + m.error.as_ref().map_or(0, String::len)
    });
    let is_running = running();
    let typing = is_running && msgs.last().is_none_or(|m| m.role == Role::User || m.parts.is_empty());
    let last_index = msgs.len().saturating_sub(1);

    rsx! {
        div { "data-scope": h::SCOPE_THREAD, "data-part": "root", "data-running": is_running.then_some(""),
            div { "data-scope": h::SCOPE_THREAD, "data-part": "scroller",
                MessageScroller { count: msgs.len(), last_key, revision, label,
                    if msgs.is_empty() {
                        p { "data-scope": h::SCOPE_THREAD, "data-part": "empty", "{empty}" }
                    }
                    for (i , m) in msgs.iter().enumerate() {
                        {
                            let (reload_id, edit_id, retry_id, fb_id, export_id, switch_id, speak_id) = (
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                            );
                            rsx! {
                                AssistantMessage {
                                    key: "{m.id}",
                                    message: m.clone(),
                                    running: is_running,
                                    is_last: i == last_index,
                                    branch: branch_map.get(&m.id).copied(),
                                    autohide,
                                    hide_when_running,
                                    on_reload: move |()| {
                                        if let Some(h) = on_reload {
                                            h.call(reload_id.clone());
                                        }
                                    },
                                    on_edit: move |()| {
                                        if let Some(h) = on_edit {
                                            h.call(edit_id.clone());
                                        }
                                    },
                                    on_retry: move |()| {
                                        if let Some(h) = on_retry {
                                            h.call(retry_id.clone());
                                        }
                                    },
                                    on_feedback: move |v| {
                                        if let Some(h) = on_feedback {
                                            h.call((fb_id.clone(), v));
                                        }
                                    },
                                    on_export: move |md| {
                                        if let Some(h) = on_export {
                                            h.call((export_id.clone(), md));
                                        }
                                    },
                                    on_branch_switch: move |i| {
                                        if let Some(h) = on_branch_switch {
                                            h.call((switch_id.clone(), i));
                                        }
                                    },
                                    on_speak: move |on| {
                                        if let Some(h) = on_speak {
                                            h.call((speak_id.clone(), on));
                                        }
                                    },
                                }
                            }
                        }
                    }
                    if typing {
                        div {
                            "data-scope": h::SCOPE_THREAD,
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
            div { "data-scope": h::SCOPE_THREAD, "data-part": "footer", {children} }
            if let Some(on_quote) = on_quote {
                AssistantSelectionToolbar { on_quote }
            }
        }
    }
}
