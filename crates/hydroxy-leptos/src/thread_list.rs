//! `<ThreadList>`: a searchable list of conversations with title, preview, time, an
//! unread marker, and inline rename and delete.

use hydroxy_headless::thread_list::{
    self as h, ThreadListCommand, ThreadListEvent, ThreadListInput, ThreadListMachine,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::prelude::*;

use crate::search_field::SearchField;

pub use hydroxy_headless::thread_list::ThreadSummary;

/// A list of conversations. The owner holds the data: apply `on_rename` and `on_delete`.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn ThreadList(
    /// The conversations, in display order.
    #[prop(into)]
    items: Signal<Vec<ThreadSummary>>,
    /// Controlled selection. Pair with `on_select`.
    #[prop(optional, into)]
    selected: MaybeProp<String>,
    /// Initially selected id when uncontrolled.
    #[prop(optional, into)]
    default_selected: Option<String>,
    /// Called with the id when a conversation is opened.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
    /// Called with the id and new title when a rename is accepted.
    #[prop(optional, into)]
    on_rename: Option<Callback<(String, String)>>,
    /// Called with the id when Delete is pressed. Show your own confirmation in the handler if needed.
    #[prop(optional, into)]
    on_delete: Option<Callback<String>>,
    /// Accessible name for the list.
    #[prop(default = "Conversations".into(), into)]
    label: String,
    /// Text shown when nothing matches.
    #[prop(default = "No conversations found.".into(), into)]
    empty: String,
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
    let send = move |event: ThreadListEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| ThreadListMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                ThreadListCommand::SelectedChange(Some(id)) => {
                    if let Some(cb) = on_select {
                        cb.run(id);
                    }
                }
                ThreadListCommand::SelectedChange(None) => {}
                ThreadListCommand::Rename { id, title } => {
                    if let Some(cb) = on_rename {
                        cb.run((id, title));
                    }
                }
                ThreadListCommand::Delete(id) => {
                    if let Some(cb) = on_delete {
                        cb.run(id);
                    }
                }
            }
        }
    };

    let visible = move || {
        let query = state.with(|s| s.query.clone());
        items.with(|all| h::filter(all, &query).into_iter().cloned().collect::<Vec<_>>())
    };

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <div data-scope=h::SCOPE data-part="search">
                <SearchField
                    label="Search conversations"
                    placeholder="Search conversations"
                    size=crate::search_field::Size::Sm
                    on_input=move |q: String| send(ThreadListEvent::SetQuery(q))
                />
            </div>
            <ul data-scope=h::SCOPE data-part="list" aria-label=label>
                <For
                    each=visible
                    key=|t| t.clone()
                    children=move |t| {
                        let ThreadSummary { id, title, preview, time, unread } = t.clone();
                        let name = h::item_label(&t);
                        let selected = Memo::new({
                            let id = id.clone();
                            move |_| state.with(|s| s.selected.as_deref() == Some(&id))
                        });
                        let renaming = Memo::new({
                            let id = id.clone();
                            move |_| state.with(|s| s.renaming.as_deref() == Some(&id))
                        });
                        let (open_id, start_id, del_id) = (id.clone(), id.clone(), id);
                        let (start_title, rename_label, delete_label) =
                            (title.clone(), format!("Rename {title}"), format!("Delete {title}"));
                        view! {
                            <li
                                data-scope=h::SCOPE
                                data-part="item"
                                data-selected=move || selected.get().then_some("")
                                data-unread=unread.then_some("")
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
                                                data-scope=h::SCOPE
                                                data-part="rename"
                                                aria-label="Conversation title"
                                                prop:value=move || state.with(|s| s.draft.clone())
                                                on:input=move |ev| send(ThreadListEvent::SetDraft(event_target_value(&ev)))
                                                on:keydown=move |ev| {
                                                    if let Some(e) = h::rename_key(&ev.key(), ev.is_composing()) {
                                                        ev.prevent_default();
                                                        send(e);
                                                    }
                                                }
                                                on:blur=move |_| send(ThreadListEvent::CommitRename)
                                            />
                                        })
                                    } else {
                                        let open = open_id.clone();
                                        Either::Right(view! {
                                            <button
                                                type="button"
                                                data-scope=h::SCOPE
                                                data-part="open"
                                                aria-label=name.clone()
                                                aria-current=move || selected.get().then_some("true")
                                                on:click=move |_| send(ThreadListEvent::Select(open.clone()))
                                            >
                                                <span data-scope=h::SCOPE data-part="marker" aria-hidden="true"></span>
                                                <span data-scope=h::SCOPE data-part="title">{title.clone()}</span>
                                                <span data-scope=h::SCOPE data-part="time">{time.clone()}</span>
                                                <span data-scope=h::SCOPE data-part="preview">{preview.clone()}</span>
                                            </button>
                                        })
                                    }
                                }}
                                <span data-scope=h::SCOPE data-part="actions">
                                    <button
                                        type="button"
                                        data-scope=h::SCOPE
                                        data-part="action"
                                        aria-label=rename_label
                                        on:click=move |_| {
                                            send(ThreadListEvent::StartRename { id: start_id.clone(), title: start_title.clone() });
                                        }
                                    >
                                        "Rename"
                                    </button>
                                    <button
                                        type="button"
                                        data-scope=h::SCOPE
                                        data-part="action"
                                        data-danger=""
                                        aria-label=delete_label
                                        on:click=move |_| send(ThreadListEvent::Delete(del_id.clone()))
                                    >
                                        "Delete"
                                    </button>
                                </span>
                            </li>
                        }
                    }
                />
            </ul>
            {move || visible().is_empty().then(|| view! { <p data-scope=h::SCOPE data-part="empty">{empty.clone()}</p> })}
        </div>
    }
}
