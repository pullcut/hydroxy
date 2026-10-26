//! `AssistantThreadList`: a New thread button over an Active and an Archived section of
//! conversations, each row with select, rename, archive and delete.

use dioxus::prelude::*;
use oxide_headless::assistant_thread::{self as h, ThreadListEntry};
use oxide_headless::thread_list::{self as tl, ThreadListCommand, ThreadListEvent, ThreadListInput, ThreadListMachine};
use oxide_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::icon::IconName;

pub use oxide_headless::assistant_thread::ThreadListEntry as Entry;

/// Conversations in an Active and an Archived section. The owner holds the data: apply
/// `on_rename`, `on_archive` and `on_delete`.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AssistantThreadList(
    /// The conversations, in display order.
    items: ReadSignal<Vec<ThreadListEntry>>,
    /// Controlled selection. Pair with `on_select`.
    #[props(default)]
    selected: ReadSignal<Option<String>>,
    /// Initially selected id when uncontrolled.
    #[props(default)]
    default_selected: Option<String>,
    /// Called with the id when a conversation is opened.
    on_select: Option<EventHandler<String>>,
    /// Called when New thread is pressed.
    on_new: Option<EventHandler<()>>,
    /// Called with the id and new title when a rename is accepted.
    on_rename: Option<EventHandler<(String, String)>>,
    /// Called with the id and whether it should now be archived.
    on_archive: Option<EventHandler<(String, bool)>>,
    /// Called with the id when Delete is pressed. Confirm in the handler if needed.
    on_delete: Option<EventHandler<String>>,
    /// Accessible name for the list.
    #[props(default = "Conversations".to_string())]
    label: String,
) -> Element {
    let input = use_memo(move || ThreadListInput {
        selected: selected().map(Some),
        default_selected: default_selected.clone(),
    });
    let mut state = use_signal(|| ThreadListMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ThreadListMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });
    let mut select = move |id: String| {
        let step = ThreadListMachine::transition(&state.peek(), ThreadListEvent::Select(id), &input.peek());
        state.set(step.state);
        for command in step.commands {
            if let (ThreadListCommand::SelectedChange(Some(id)), Some(handler)) = (command, on_select) {
                handler.call(id);
            }
        }
    };

    let all = items.read();
    let (active, archived) = h::split_archived(&all);
    let current = state.read().selected.clone();
    let sections = [("Active", false, active), ("Archived", true, archived)];

    rsx! {
        nav { "data-scope": h::SCOPE_THREAD_LIST, "data-part": "root", "aria-label": "{label}",
            div { "data-scope": h::SCOPE_THREAD_LIST, "data-part": "new",
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    icon_left: IconName::Plus,
                    on_click: move |_| {
                        if let Some(handler) = on_new {
                            handler.call(());
                        }
                    },
                    "New thread"
                }
            }
            for (name , is_archived , rows) in sections {
                if !is_archived || !rows.is_empty() {
                    section {
                        key: "{name}",
                        "data-scope": h::SCOPE_THREAD_LIST,
                        "data-part": "section",
                        "data-archived": is_archived.then_some(""),
                        h3 { "data-scope": h::SCOPE_THREAD_LIST, "data-part": "heading", "{name}" }
                        ul { "data-scope": h::SCOPE_THREAD_LIST, "data-part": "list", "aria-label": "{name}",
                            for e in rows {
                                {
                                    let (open_id, rename_id, archive_id, delete_id) = (
                                        e.id.clone(),
                                        e.id.clone(),
                                        e.id.clone(),
                                        e.id.clone(),
                                    );
                                    rsx! {
                                        AssistantThreadListItem {
                                            key: "{e.id}",
                                            entry: e.clone(),
                                            selected: current.as_deref() == Some(e.id.as_str()),
                                            on_select: move |()| select(open_id.clone()),
                                            on_rename: move |t: String| {
                                                if let Some(handler) = on_rename {
                                                    handler.call((rename_id.clone(), t));
                                                }
                                            },
                                            on_archive: move |()| {
                                                if let Some(handler) = on_archive {
                                                    handler.call((archive_id.clone(), !is_archived));
                                                }
                                            },
                                            on_delete: move |()| {
                                                if let Some(handler) = on_delete {
                                                    handler.call(delete_id.clone());
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if all.is_empty() {
                p { "data-scope": h::SCOPE_THREAD_LIST, "data-part": "empty", "No conversations yet." }
            }
        }
    }
}

/// One conversation row: an open button with the title, and Rename, Archive (or Restore)
/// and Delete on hover or focus. Rename edits in place with Enter to save, Escape to cancel.
#[component]
pub fn AssistantThreadListItem(
    /// The conversation.
    entry: ThreadListEntry,
    /// This is the open conversation.
    #[props(default)]
    selected: bool,
    /// Called when the row is opened.
    on_select: Option<EventHandler<()>>,
    /// Called with the new title when a rename is accepted.
    on_rename: Option<EventHandler<String>>,
    /// Called when Archive or Restore is pressed.
    on_archive: Option<EventHandler<()>>,
    /// Called when Delete is pressed.
    on_delete: Option<EventHandler<()>>,
) -> Element {
    let mut state = use_signal(|| ThreadListMachine::initial(&ThreadListInput::default()));
    let mut send = move |event: ThreadListEvent| {
        let step = ThreadListMachine::transition(&state.peek(), event, &ThreadListInput::default());
        state.set(step.state);
        for command in step.commands {
            if let (ThreadListCommand::Rename { title, .. }, Some(handler)) = (command, on_rename) {
                handler.call(title);
            }
        }
    };
    let ThreadListEntry { id, title, archived } = entry;
    let renaming = state.read().renaming.as_deref() == Some(id.as_str());
    let (start_id, start_title) = (id.clone(), title.clone());
    let archive_label = format!("{} {title}", if archived { "Restore" } else { "Archive" });
    rsx! {
        li {
            "data-scope": h::SCOPE_THREAD_LIST_ITEM,
            "data-part": "root",
            "data-selected": selected.then_some(""),
            "data-archived": archived.then_some(""),
            if renaming {
                input {
                    "data-scope": h::SCOPE_THREAD_LIST_ITEM,
                    "data-part": "rename",
                    "aria-label": "Conversation title",
                    value: "{state.read().draft}",
                    onmounted: move |e| {
                        let el = e.data();
                        spawn(async move {
                            let _ = el.set_focus(true).await;
                        });
                    },
                    oninput: move |e| send(ThreadListEvent::SetDraft(e.value())),
                    onkeydown: move |e| {
                        let key = e.key().to_string();
                        if let Some(event) = tl::rename_key(&key, e.is_composing()) {
                            e.prevent_default();
                            send(event);
                        }
                    },
                    onblur: move |_| send(ThreadListEvent::CommitRename),
                }
            } else {
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE_THREAD_LIST_ITEM,
                    "data-part": "open",
                    "aria-current": selected.then_some("true"),
                    onclick: move |_| {
                        if let Some(handler) = on_select {
                            handler.call(());
                        }
                    },
                    "{title}"
                }
            }
            span { "data-scope": h::SCOPE_THREAD_LIST_ITEM, "data-part": "actions",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE_THREAD_LIST_ITEM,
                    "data-part": "action",
                    "aria-label": "Rename {title}",
                    onclick: move |_| {
                        send(ThreadListEvent::StartRename { id: start_id.clone(), title: start_title.clone() });
                    },
                    "Rename"
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE_THREAD_LIST_ITEM,
                    "data-part": "action",
                    "aria-label": archive_label,
                    onclick: move |_| {
                        if let Some(handler) = on_archive {
                            handler.call(());
                        }
                    },
                    if archived {
                        "Restore"
                    } else {
                        "Archive"
                    }
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE_THREAD_LIST_ITEM,
                    "data-part": "action",
                    "data-danger": "",
                    "aria-label": "Delete {title}",
                    onclick: move |_| {
                        if let Some(handler) = on_delete {
                            handler.call(());
                        }
                    },
                    "Delete"
                }
            }
        }
    }
}
