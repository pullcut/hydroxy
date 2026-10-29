//! `<AssistantThreadList>`: a New thread button over an Active and an Archived section of
//! conversations, each row with select, rename, archive and delete.

use hydroxy_headless::assistant_thread::{self as h, ThreadListEntry};
use hydroxy_headless::thread_list::{
    self as tl, ThreadListCommand, ThreadListEvent, ThreadListInput, ThreadListMachine,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::prelude::*;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::drive::drive;
use crate::icon::IconName;

pub use hydroxy_headless::assistant_thread::ThreadListEntry as Entry;

/// Conversations in an Active and an Archived section. The owner holds the data: apply
/// `on_rename`, `on_archive` and `on_delete`.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn AssistantThreadList(
    /// The conversations, in display order.
    #[prop(into)]
    items: Signal<Vec<ThreadListEntry>>,
    /// Controlled selection. Pair with `on_select`.
    #[prop(optional, into)]
    selected: MaybeProp<String>,
    /// Initially selected id when uncontrolled.
    #[prop(optional, into)]
    default_selected: Option<String>,
    /// Called with the id when a conversation is opened.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
    /// Called when New thread is pressed.
    #[prop(optional, into)]
    on_new: Option<Callback<()>>,
    /// Called with the id and new title when a rename is accepted.
    #[prop(optional, into)]
    on_rename: Option<Callback<(String, String)>>,
    /// Called with the id and whether it should now be archived.
    #[prop(optional, into)]
    on_archive: Option<Callback<(String, bool)>>,
    /// Called with the id when Delete is pressed. Confirm in the handler if needed.
    #[prop(optional, into)]
    on_delete: Option<Callback<String>>,
    /// Accessible name for the list.
    #[prop(default = "Conversations".into(), into)]
    label: String,
) -> impl IntoView {
    let input = Memo::new(move |_| ThreadListInput {
        selected: selected.get().map(Some),
        default_selected: default_selected.clone(),
    });
    let state = RwSignal::new(input.with_untracked(ThreadListMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| ThreadListMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let select = move |id: String| {
        for command in drive::<ThreadListMachine, _>(state, input, ThreadListEvent::Select(id)) {
            if let (ThreadListCommand::SelectedChange(Some(id)), Some(cb)) = (command, on_select) {
                cb.run(id);
            }
        }
    };
    let split = Memo::new(move |_| {
        items.with(|all| {
            let (a, r) = h::split_archived(all);
            (a.into_iter().cloned().collect::<Vec<_>>(), r.into_iter().cloned().collect::<Vec<_>>())
        })
    });

    let section = move |name: &'static str, archived: bool| {
        let rows = move || split.with(|s| if archived { s.1.clone() } else { s.0.clone() });
        view! {
            <section data-scope=h::SCOPE_THREAD_LIST data-part="section" data-archived=archived.then_some("")>
                <h3 data-scope=h::SCOPE_THREAD_LIST data-part="heading">{name}</h3>
                <ul data-scope=h::SCOPE_THREAD_LIST data-part="list" aria-label=name>
                    <For
                        each=rows
                        key=|e| (e.id.clone(), e.title.clone(), e.archived)
                        children=move |e| {
                            let id = e.id.clone();
                            let is_selected = Signal::derive({
                                let id = id.clone();
                                move || state.with(|s| s.selected.value.as_deref() == Some(&id))
                            });
                            let (open_id, rename_id, archive_id, delete_id) = (id.clone(), id.clone(), id.clone(), id);
                            let archived = e.archived;
                            view! {
                                <AssistantThreadListItem
                                    entry=e
                                    selected=is_selected
                                    on_select=move |()| select(open_id.clone())
                                    on_rename=move |t: String| {
                                        if let Some(cb) = on_rename {
                                            cb.run((rename_id.clone(), t));
                                        }
                                    }
                                    on_archive=move |()| {
                                        if let Some(cb) = on_archive {
                                            cb.run((archive_id.clone(), !archived));
                                        }
                                    }
                                    on_delete=move |()| {
                                        if let Some(cb) = on_delete {
                                            cb.run(delete_id.clone());
                                        }
                                    }
                                />
                            }
                        }
                    />
                </ul>
            </section>
        }
    };

    view! {
        <nav data-scope=h::SCOPE_THREAD_LIST data-part="root" aria-label=label>
            <div data-scope=h::SCOPE_THREAD_LIST data-part="new">
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Sm
                    icon_left=IconName::Plus
                    on_click=move |_| {
                        if let Some(cb) = on_new {
                            cb.run(());
                        }
                    }
                >
                    "New thread"
                </Button>
            </div>
            {section("Active", false)}
            {move || split.with(|s| !s.1.is_empty()).then(|| section("Archived", true))}
            {move || {
                items
                    .with(Vec::is_empty)
                    .then(|| view! { <p data-scope=h::SCOPE_THREAD_LIST data-part="empty">"No conversations yet."</p> })
            }}
        </nav>
    }
}

/// One conversation row: an open button with the title, and Rename, Archive (or Restore)
/// and Delete on hover or focus. Rename edits in place with Enter to save, Escape to cancel.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn AssistantThreadListItem(
    /// The conversation.
    #[prop(into)]
    entry: ThreadListEntry,
    /// This is the open conversation.
    #[prop(optional, into)]
    selected: Signal<bool>,
    /// Called when the row is opened.
    #[prop(optional, into)]
    on_select: Option<Callback<()>>,
    /// Called with the new title when a rename is accepted.
    #[prop(optional, into)]
    on_rename: Option<Callback<String>>,
    /// Called when Archive or Restore is pressed.
    #[prop(optional, into)]
    on_archive: Option<Callback<()>>,
    /// Called when Delete is pressed.
    #[prop(optional, into)]
    on_delete: Option<Callback<()>>,
) -> impl IntoView {
    let input = Signal::stored(ThreadListInput::default());
    let state = RwSignal::new(input.with_untracked(ThreadListMachine::initial));
    let send = move |event: ThreadListEvent| {
        for command in drive::<ThreadListMachine, _>(state, input, event) {
            if let (ThreadListCommand::Rename { title, .. }, Some(cb)) = (command, on_rename) {
                cb.run(title);
            }
        }
    };
    let ThreadListEntry { id, title, archived } = entry;
    let renaming = Memo::new({
        let id = id.clone();
        move |_| state.with(|s| s.renaming.as_deref() == Some(&id))
    });
    let (start_title, rename_label, delete_label, archive_label) = (
        title.clone(),
        format!("Rename {title}"),
        format!("Delete {title}"),
        format!("{} {title}", if archived { "Restore" } else { "Archive" }),
    );
    view! {
        <li
            data-scope=h::SCOPE_THREAD_LIST_ITEM
            data-part="root"
            data-selected=move || selected.get().then_some("")
            data-archived=archived.then_some("")
        >
            {move || {
                if renaming.get() {
                    let field = NodeRef::<leptos::html::Input>::new();
                    Effect::new(move |_| {
                        if let Some(el) = field.get() {
                            let _ = el.focus();
                            el.select();
                        }
                    });
                    Either::Left(view! {
                        <input
                            node_ref=field
                            data-scope=h::SCOPE_THREAD_LIST_ITEM
                            data-part="rename"
                            aria-label="Conversation title"
                            prop:value=move || state.with(|s| s.draft.clone())
                            on:input=move |ev| send(ThreadListEvent::SetDraft(event_target_value(&ev)))
                            on:keydown=move |ev| {
                                if let Some(e) = tl::rename_key(&ev.key(), ev.is_composing()) {
                                    ev.prevent_default();
                                    send(e);
                                }
                            }
                            on:blur=move |_| send(ThreadListEvent::CommitRename)
                        />
                    })
                } else {
                    Either::Right(view! {
                        <button
                            type="button"
                            data-scope=h::SCOPE_THREAD_LIST_ITEM
                            data-part="open"
                            aria-current=move || selected.get().then_some("true")
                            on:click=move |_| {
                                if let Some(cb) = on_select {
                                    cb.run(());
                                }
                            }
                        >
                            {title.clone()}
                        </button>
                    })
                }
            }}
            <span data-scope=h::SCOPE_THREAD_LIST_ITEM data-part="actions">
                <button
                    type="button"
                    data-scope=h::SCOPE_THREAD_LIST_ITEM
                    data-part="action"
                    aria-label=rename_label
                    on:click=move |_| {
                        send(ThreadListEvent::StartRename { id: id.clone(), title: start_title.clone() });
                    }
                >
                    "Rename"
                </button>
                <button
                    type="button"
                    data-scope=h::SCOPE_THREAD_LIST_ITEM
                    data-part="action"
                    aria-label=archive_label
                    on:click=move |_| {
                        if let Some(cb) = on_archive {
                            cb.run(());
                        }
                    }
                >
                    {if archived { "Restore" } else { "Archive" }}
                </button>
                <button
                    type="button"
                    data-scope=h::SCOPE_THREAD_LIST_ITEM
                    data-part="action"
                    data-danger=""
                    aria-label=delete_label
                    on:click=move |_| {
                        if let Some(cb) = on_delete {
                            cb.run(());
                        }
                    }
                >
                    "Delete"
                </button>
            </span>
        </li>
    }
}
