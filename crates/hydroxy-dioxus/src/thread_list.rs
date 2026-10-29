//! `ThreadList`: a searchable list of conversations with title, preview, time, an
//! unread marker, and inline rename and delete.

use dioxus::prelude::*;
use hydroxy_headless::thread_list::{
    self as h, ThreadListCommand, ThreadListEvent, ThreadListInput, ThreadListMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::search_field::{SearchField, Size};

pub use hydroxy_headless::thread_list::ThreadSummary;

/// A list of conversations. The owner holds the data: apply `on_rename` and `on_delete`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn ThreadList(
    /// The conversations, in display order.
    items: ReadSignal<Vec<ThreadSummary>>,
    /// Controlled selection. Pair with `on_select`.
    #[props(default)]
    selected: ReadSignal<Option<String>>,
    /// Initially selected id when uncontrolled.
    #[props(default)]
    default_selected: Option<String>,
    /// Called with the id when a conversation is opened.
    on_select: Option<EventHandler<String>>,
    /// Called with the id and new title when a rename is accepted.
    on_rename: Option<EventHandler<(String, String)>>,
    /// Called with the id when Delete is pressed. Show your own confirmation in the handler if needed.
    on_delete: Option<EventHandler<String>>,
    /// Accessible name for the list.
    #[props(default = "Conversations".to_string())]
    label: String,
    /// Text shown when nothing matches.
    #[props(default = "No conversations found.".to_string())]
    empty: String,
) -> Element {
    let input = use_memo(move || ThreadListInput {
        selected: selected().map(Some),
        default_selected: default_selected.clone(),
    });
    let mut state = use_signal(|| ThreadListMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ThreadListMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let send = move |event: ThreadListEvent| {
        for command in drive::<ThreadListMachine>(state, input, event) {
            match command {
                ThreadListCommand::SelectedChange(Some(id)) => {
                    if let Some(handler) = on_select {
                        handler.call(id);
                    }
                }
                ThreadListCommand::SelectedChange(None) => {}
                ThreadListCommand::Rename { id, title } => {
                    if let Some(handler) = on_rename {
                        handler.call((id, title));
                    }
                }
                ThreadListCommand::Delete(id) => {
                    if let Some(handler) = on_delete {
                        handler.call(id);
                    }
                }
            }
        }
    };

    let s = state.read();
    let all = items.read();
    let visible = h::filter(&all, &s.query);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            div { "data-scope": h::SCOPE, "data-part": "search",
                SearchField {
                    label: "Search conversations",
                    placeholder: "Search conversations",
                    size: Size::Sm,
                    on_input: move |q: String| send(ThreadListEvent::SetQuery(q)),
                }
            }
            ul { "data-scope": h::SCOPE, "data-part": "list", "aria-label": "{label}",
                for t in visible.iter() {
                    {
                        let id = t.id.clone();
                        let is_selected = s.selected.value.as_deref() == Some(t.id.as_str());
                        let renaming = s.renaming.as_deref() == Some(t.id.as_str());
                        let (open_id, start_id, del_id) = (id.clone(), id.clone(), id.clone());
                        let start_title = t.title.clone();
                        rsx! {
                            li {
                                key: "{id}",
                                "data-scope": h::SCOPE,
                                "data-part": "item",
                                "data-selected": is_selected.then_some(""),
                                "data-unread": t.unread.then_some(""),
                                if renaming {
                                    input {
                                        "data-scope": h::SCOPE,
                                        "data-part": "rename",
                                        "aria-label": "Conversation title",
                                        value: "{s.draft}",
                                        onmounted: move |e| {
                                            let el = e.data();
                                            spawn(async move {
                                                let _ = el.set_focus(true).await;
                                            });
                                        },
                                        oninput: move |e| send(ThreadListEvent::SetDraft(e.value())),
                                        onkeydown: move |e| {
                                            let key = e.key().to_string();
                                            if let Some(event) = h::rename_key(&key, e.is_composing()) {
                                                e.prevent_default();
                                                send(event);
                                            }
                                        },
                                        onblur: move |_| send(ThreadListEvent::CommitRename),
                                    }
                                } else {
                                    button {
                                        r#type: "button",
                                        "data-scope": h::SCOPE,
                                        "data-part": "open",
                                        "aria-label": h::item_label(t),
                                        "aria-current": is_selected.then_some("true"),
                                        onclick: move |_| send(ThreadListEvent::Select(open_id.clone())),
                                        span { "data-scope": h::SCOPE, "data-part": "marker", "aria-hidden": "true" }
                                        span { "data-scope": h::SCOPE, "data-part": "title", "{t.title}" }
                                        span { "data-scope": h::SCOPE, "data-part": "time", "{t.time}" }
                                        span { "data-scope": h::SCOPE, "data-part": "preview", "{t.preview}" }
                                    }
                                }
                                span { "data-scope": h::SCOPE, "data-part": "actions",
                                    button {
                                        r#type: "button",
                                        "data-scope": h::SCOPE,
                                        "data-part": "action",
                                        "aria-label": "Rename {t.title}",
                                        onclick: move |_| {
                                            send(ThreadListEvent::StartRename {
                                                id: start_id.clone(),
                                                title: start_title.clone(),
                                            });
                                        },
                                        "Rename"
                                    }
                                    button {
                                        r#type: "button",
                                        "data-scope": h::SCOPE,
                                        "data-part": "action",
                                        "data-danger": "",
                                        "aria-label": "Delete {t.title}",
                                        onclick: move |_| send(ThreadListEvent::Delete(del_id.clone())),
                                        "Delete"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if visible.is_empty() {
                p { "data-scope": h::SCOPE, "data-part": "empty", "{empty}" }
            }
        }
    }
}
