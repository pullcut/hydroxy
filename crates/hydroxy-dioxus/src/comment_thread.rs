//! `CommentThread`: threaded comments with replies, resolve and reopen, avatars,
//! relative times and a reply composer.

use dioxus::prelude::*;
use hydroxy_headless::comment_thread::{
    self as h, Comment, CommentThreadCommand, CommentThreadEvent, CommentThreadInput, CommentThreadMachine,
};
use hydroxy_statechart::Machine;

use crate::avatar::Avatar;
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::composer::Composer;

pub use hydroxy_headless::comment_thread::{Comment as CommentData, Jewel, relative_time};

fn comment_view(c: &Comment, now: i64, reply: bool) -> Element {
    rsx! {
        article {
            "data-scope": h::SCOPE,
            "data-part": "comment",
            "data-reply": reply.then_some(""),
            Avatar { initials: c.initials.clone(), jewel: c.tint, size: 26, title: c.author.clone() }
            div { "data-scope": h::SCOPE, "data-part": "content",
                header { "data-scope": h::SCOPE, "data-part": "head",
                    b { "data-scope": h::SCOPE, "data-part": "author", "{c.author}" }
                    span { "data-scope": h::SCOPE, "data-part": "time", "{h::relative_time(now, c.created)}" }
                }
                p { "data-scope": h::SCOPE, "data-part": "body", "{c.body}" }
            }
        }
    }
}

/// Threads of comments. The owner holds the comments: append in `on_reply` and `on_comment`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn CommentThread(
    /// Top-level comments, each carrying its replies.
    comments: ReadSignal<Vec<Comment>>,
    /// The current time in unix seconds, for relative times.
    now: ReadSignal<i64>,
    /// Controlled resolved thread ids. Pair with `on_resolved_change`.
    #[props(default)]
    resolved: ReadSignal<Option<Vec<String>>>,
    /// Initially resolved thread ids when uncontrolled.
    #[props(default)]
    default_resolved: Vec<String>,
    /// Called with the thread id and whether it is now resolved.
    on_resolved_change: Option<EventHandler<(String, bool)>>,
    /// Called with the thread id and the reply text.
    on_reply: Option<EventHandler<(String, String)>>,
    /// Called with the text of a new thread. Shows a composer under the threads when set.
    on_comment: Option<EventHandler<String>>,
    /// Accessible name for the list.
    #[props(default = "Comments".to_string())]
    label: String,
) -> Element {
    let input =
        use_memo(move || CommentThreadInput { resolved: resolved(), default_resolved: default_resolved.clone() });
    let mut state = use_signal(|| CommentThreadMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CommentThreadMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root = use_signal(|| hydroxy_headless::next_id(h::SCOPE));
    let mut send = move |event: CommentThreadEvent| {
        let step = CommentThreadMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                CommentThreadCommand::ResolvedChange { id, resolved } => {
                    if let Some(handler) = on_resolved_change {
                        handler.call((id, resolved));
                    }
                }
                CommentThreadCommand::FocusReply(id) => {
                    let wrapper = format!("{}-reply-{id}", root.peek());
                    spawn(async move {
                        // The box renders after this event; wait a tick, then focus its field.
                        // `{wrapper:?}` quotes the id as a JS string, so ids with quotes are safe
                        // as long as they have no characters Rust escapes differently from JS.
                        let _ = document::eval(&format!(
                            "await new Promise((r) => setTimeout(r, 0));
                            document.getElementById({wrapper:?})?.querySelector('textarea')?.focus();"
                        ))
                        .await;
                    });
                }
            }
        }
    };

    let s = state.read();
    let now = now();
    let all = comments.read();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            ol { "data-scope": h::SCOPE, "data-part": "threads", "aria-label": "{label}",
                for thread in all.iter() {
                    {
                        let id = thread.id.clone();
                        let is_resolved = s.is_resolved(&id);
                        let open = s.is_open(&id);
                        let expanded = open && is_resolved;
                        let replying = s.replying.as_deref() == Some(id.as_str());
                        let (toggle_id, reopen_id, resolve_id, reply_open_id, sent_id) =
                            (id.clone(), id.clone(), id.clone(), id.clone(), id.clone());
                        let wrapper = format!("{}-reply-{id}", root.read());
                        rsx! {
                            li {
                                key: "{id}",
                                "data-scope": h::SCOPE,
                                "data-part": "thread",
                                "data-state": h::thread_state(&s, &id),
                                if is_resolved {
                                    div { "data-scope": h::SCOPE, "data-part": "resolved",
                                        span { "data-scope": h::SCOPE, "data-part": "summary",
                                            "{h::resolved_summary(&thread.author, thread.replies.len())}"
                                        }
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            size: ButtonSize::Sm,
                                            on_click: move |_| send(CommentThreadEvent::ToggleExpanded(toggle_id.clone())),
                                            if expanded {
                                                "Hide"
                                            } else {
                                                "Show"
                                            }
                                        }
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            size: ButtonSize::Sm,
                                            on_click: move |_| send(CommentThreadEvent::Reopen(reopen_id.clone())),
                                            "Reopen"
                                        }
                                    }
                                }
                                if open {
                                    div { "data-scope": h::SCOPE, "data-part": "comments",
                                        {comment_view(thread, now, false)}
                                        div { "data-scope": h::SCOPE, "data-part": "replies",
                                            for r in thread.replies.iter() {
                                                {comment_view(r, now, true)}
                                            }
                                        }
                                        if !is_resolved {
                                            div { "data-scope": h::SCOPE, "data-part": "actions",
                                                Button {
                                                    variant: ButtonVariant::Ghost,
                                                    size: ButtonSize::Sm,
                                                    on_click: move |_| send(CommentThreadEvent::OpenReply(reply_open_id.clone())),
                                                    "Reply"
                                                }
                                                Button {
                                                    variant: ButtonVariant::Ghost,
                                                    size: ButtonSize::Sm,
                                                    on_click: move |_| send(CommentThreadEvent::Resolve(resolve_id.clone())),
                                                    "Resolve"
                                                }
                                            }
                                        }
                                    }
                                }
                                if replying && !is_resolved {
                                    div {
                                        id: "{wrapper}",
                                        "data-scope": h::SCOPE,
                                        "data-part": "reply",
                                        onkeydown: move |e| {
                                            if e.key() == Key::Escape {
                                                e.stop_propagation();
                                                send(CommentThreadEvent::CloseReply);
                                            }
                                        },
                                        Composer {
                                            placeholder: "Write a reply",
                                            label: "Reply",
                                            on_send: move |text: String| {
                                                if let Some(handler) = on_reply {
                                                    handler.call((sent_id.clone(), text));
                                                }
                                                send(CommentThreadEvent::CloseReply);
                                            },
                                        }
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            size: ButtonSize::Sm,
                                            on_click: move |_| send(CommentThreadEvent::CloseReply),
                                            "Cancel"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(handler) = on_comment {
                div { "data-scope": h::SCOPE, "data-part": "new",
                    Composer {
                        placeholder: "Add a comment",
                        label: "New comment",
                        on_send: move |t: String| handler.call(t),
                    }
                }
            }
        }
    }
}
