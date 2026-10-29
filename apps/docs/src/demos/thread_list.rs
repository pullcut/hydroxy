use hydroxy_leptos::thread_list::{ThreadList, ThreadSummary};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (items, set_items) = signal(vec![
        ThreadSummary::new("churn", "Q3 churn drivers", "Pricing came up in nine of fourteen interviews", "2m")
            .unread(),
        ThreadSummary::new("board", "Board update sources", "Checked every figure against the dashboard", "1h"),
        ThreadSummary::new(
            "digest",
            "Weekly digest draft",
            "Here is a shorter version for the newsletter",
            "Yesterday",
        ),
        ThreadSummary::new(
            "onboarding",
            "Onboarding survey themes",
            "Setup calls mattered most for small teams",
            "24 Oct",
        ),
        ThreadSummary::new("pricing", "Pricing page copy", "Three options for the annual plan headline", "21 Oct"),
    ]);
    let (selected, set_selected) = signal(Some("board".to_string()));

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 360px;">
            <ThreadList
                items=items
                selected=selected
                on_select=move |id: String| {
                    // Opening a conversation reads it.
                    set_items.update(|v| v.iter_mut().filter(|t| t.id == id).for_each(|t| t.unread = false));
                    set_selected.set(Some(id));
                }
                on_rename=move |(id, title): (String, String)| {
                    set_items.update(|v| v.iter_mut().filter(|t| t.id == id).for_each(|t| t.title.clone_from(&title)));
                }
                on_delete=move |id: String| set_items.update(|v| v.retain(|t| t.id != id))
            />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || match selected.get() {
                    Some(id) => format!("Open: {id}"),
                    None => "Nothing open.".to_string(),
                }}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "thread-list",
    name: "Thread list",
    group: "Chat",
    summary: "The conversation sidebar. Each row shows a title, preview and time, with an unread marker. Search filters as you type; rename and delete sit on each row.",
    demo,
    leptos: include_str!("thread_list.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/thread_list.rs"),
    parts: &[
        &["ThreadList", "root", "Search field above the list"],
        &["", "search", "Wraps the SearchField"],
        &["", "list", "ul of conversations"],
        &["", "item", "A row. data-selected, data-unread"],
        &["", "open", "The row's button. aria-current on the open conversation"],
        &["", "marker / title / time / preview", "Contents of the open button"],
        &["", "actions", "Rename and Delete, shown on hover or focus"],
        &["", "rename", "Text field while renaming"],
        &["", "empty", "Shown when no conversation matches"],
    ],
    keys: &[
        &["Enter", "In the rename field: save the new title"],
        &["Escape", "In the rename field: leave the title as it was"],
        &["Escape", "In the search field: clear the search"],
    ],
    props: &[
        &[
            "items",
            "Vec<ThreadSummary>",
            "Conversations in display order. A summary has id, title, preview, time and unread",
        ],
        &["selected", "Option<String>", "Controlled open id. Pair with on_select"],
        &["default_selected", "String", "Open id when uncontrolled"],
        &["on_select", "Callback<String>", "Called with the id when a row is opened"],
        &["on_rename", "Callback<(String, String)>", "Id and trimmed new title. You apply it"],
        &["on_delete", "Callback<String>", "Id to remove. You apply it, and can confirm first"],
        &["label / empty", "String", "Accessible name and empty text"],
    ],
    events: &[],
};
