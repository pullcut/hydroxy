//! `<CommentThread>`: threaded comments with replies, resolve and reopen, avatars,
//! relative times and a reply composer.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::comment_thread::{
    self as h, Comment, CommentThreadCommand, CommentThreadEvent, CommentThreadInput, CommentThreadMachine,
};
use oxide_statechart::Machine;

use crate::avatar::Avatar;
use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::composer::Composer;

pub use oxide_headless::comment_thread::{Comment as CommentData, Jewel, relative_time};

fn comment_view(c: &Comment, now: Signal<i64>, reply: bool) -> impl IntoView + use<> {
    let (created, name) = (c.created, c.author.clone());
    view! {
        <article data-scope=h::SCOPE data-part="comment" data-reply=reply.then_some("")>
            <Avatar initials=c.initials.clone() jewel=c.tint size=26 title=name.clone() />
            <div data-scope=h::SCOPE data-part="content">
                <header data-scope=h::SCOPE data-part="head">
                    <b data-scope=h::SCOPE data-part="author">{name}</b>
                    <span data-scope=h::SCOPE data-part="time">{move || h::relative_time(now.get(), created)}</span>
                </header>
                <p data-scope=h::SCOPE data-part="body">{c.body.clone()}</p>
            </div>
        </article>
    }
}

/// Threads of comments. The owner holds the comments: append in `on_reply` and `on_comment`.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn CommentThread(
    /// Top-level comments, each carrying its replies.
    #[prop(into)]
    comments: Signal<Vec<Comment>>,
    /// The current time in unix seconds, for relative times. Pass it in so server and
    /// browser render the same text.
    #[prop(into)]
    now: Signal<i64>,
    /// Controlled resolved thread ids. Pair with `on_resolved_change`.
    #[prop(optional, into)]
    resolved: MaybeProp<Vec<String>>,
    /// Initially resolved thread ids when uncontrolled.
    #[prop(optional, into)]
    default_resolved: Vec<String>,
    /// Called with the thread id and whether it is now resolved.
    #[prop(optional, into)]
    on_resolved_change: Option<Callback<(String, bool)>>,
    /// Called with the thread id and the reply text.
    #[prop(optional, into)]
    on_reply: Option<Callback<(String, String)>>,
    /// Called with the text of a new thread. Shows a composer under the threads when set.
    #[prop(optional, into)]
    on_comment: Option<Callback<String>>,
    /// Accessible name for the list.
    #[prop(default = "Comments".into(), into)]
    label: String,
) -> impl IntoView {
    let input = Memo::new(move |_| CommentThreadInput { resolved: resolved.get(), default_resolved: default_resolved.clone() });
    let state = RwSignal::new(input.with_untracked(CommentThreadMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CommentThreadMachine::sync(s, i)));
        state.set(synced);
    });
    let root_id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let send = move |event: CommentThreadEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| CommentThreadMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        state.set(step.state);
        for command in step.commands {
            match command {
                CommentThreadCommand::ResolvedChange { id, resolved } => {
                    if let Some(cb) = on_resolved_change {
                        cb.run((id, resolved));
                    }
                }
                CommentThreadCommand::FocusReply(id) => {
                    let wrapper = reply_id(&root_id.get_value(), &id);
                    // The box renders after this event; focus its field once the DOM has caught up.
                    set_timeout(
                        move || {
                            let field = document()
                                .get_element_by_id(&wrapper)
                                .and_then(|w| w.query_selector("textarea").ok().flatten())
                                .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok());
                            if let Some(f) = field {
                                let _ = f.focus();
                            }
                        },
                        std::time::Duration::ZERO,
                    );
                }
            }
        }
    };

    // the whole list re-renders when `comments` changes; key threads if lists get long.
    let threads = move || {
        comments
            .get()
            .into_iter()
            .map(|thread| {
                let id = thread.id.clone();
                let rid = reply_id(&root_id.get_value(), &id);
                let state_id = id.clone();
                let body = move || {
                    let (id, thread) = (id.clone(), thread.clone());
                    let (open, is_resolved, replying) = state.with(|s| {
                        (s.is_open(&id), s.is_resolved(&id), s.replying.as_deref() == Some(&id))
                    });
                    let summary = h::resolved_summary(&thread.author, thread.replies.len());
                    let (toggle_id, reopen_id, resolve_id, reply_open_id, sent_id) =
                        (id.clone(), id.clone(), id.clone(), id.clone(), id.clone());
                    let expanded = open && is_resolved;
                    view! {
                        {(is_resolved)
                            .then(|| {
                                view! {
                                    <div data-scope=h::SCOPE data-part="resolved">
                                        <span data-scope=h::SCOPE data-part="summary">{summary}</span>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            size=ButtonSize::Sm
                                            on_click=move |_| send(CommentThreadEvent::ToggleExpanded(toggle_id.clone()))
                                        >
                                            {if expanded { "Hide" } else { "Show" }}
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            size=ButtonSize::Sm
                                            on_click=move |_| send(CommentThreadEvent::Reopen(reopen_id.clone()))
                                        >
                                            "Reopen"
                                        </Button>
                                    </div>
                                }
                            })}
                        {open
                            .then(|| {
                                view! {
                                    <div data-scope=h::SCOPE data-part="comments">
                                        {comment_view(&thread, now, false)}
                                        <div data-scope=h::SCOPE data-part="replies">
                                            {thread.replies.iter().map(|r| comment_view(r, now, true)).collect_view()}
                                        </div>
                                        {(!is_resolved)
                                            .then(|| {
                                                view! {
                                                    <div data-scope=h::SCOPE data-part="actions">
                                                        <Button
                                                            variant=ButtonVariant::Ghost
                                                            size=ButtonSize::Sm
                                                            on_click=move |_| send(CommentThreadEvent::OpenReply(reply_open_id.clone()))
                                                        >
                                                            "Reply"
                                                        </Button>
                                                        <Button
                                                            variant=ButtonVariant::Ghost
                                                            size=ButtonSize::Sm
                                                            on_click=move |_| send(CommentThreadEvent::Resolve(resolve_id.clone()))
                                                        >
                                                            "Resolve"
                                                        </Button>
                                                    </div>
                                                }
                                            })}
                                    </div>
                                }
                            })}
                        {(replying && !is_resolved)
                            .then(|| {
                                let wrapper = rid.clone();
                                view! {
                                    <div
                                        id=wrapper
                                        data-scope=h::SCOPE
                                        data-part="reply"
                                        on:keydown=move |ev| {
                                            if ev.key() == "Escape" {
                                                ev.stop_propagation();
                                                send(CommentThreadEvent::CloseReply);
                                            }
                                        }
                                    >
                                        <Composer
                                            placeholder="Write a reply"
                                            label="Reply"
                                            on_send=move |text: String| {
                                                if let Some(cb) = on_reply {
                                                    cb.run((sent_id.clone(), text));
                                                }
                                                send(CommentThreadEvent::CloseReply);
                                            }
                                        />
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            size=ButtonSize::Sm
                                            on_click=move |_| send(CommentThreadEvent::CloseReply)
                                        >
                                            "Cancel"
                                        </Button>
                                    </div>
                                }
                            })}
                    }
                };
                view! {
                    <li data-scope=h::SCOPE data-part="thread" data-state=move || state.with(|s| h::thread_state(s, &state_id))>
                        {body}
                    </li>
                }
            })
            .collect_view()
    };

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <ol data-scope=h::SCOPE data-part="threads" aria-label=label>{threads}</ol>
            {on_comment
                .map(|cb| {
                    view! {
                        <div data-scope=h::SCOPE data-part="new">
                            <Composer placeholder="Add a comment" label="New comment" on_send=move |t: String| cb.run(t) />
                        </div>
                    }
                })}
        </div>
    }
}

fn reply_id(root: &str, thread: &str) -> String {
    format!("{root}-reply-{thread}")
}
