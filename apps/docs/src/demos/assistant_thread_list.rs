use leptos::prelude::*;
use oxide_leptos::assistant_thread_list::{AssistantThreadList, Entry};

pub fn demo() -> AnyView {
    let (items, set_items) = signal(vec![
        Entry::new("churn", "Q3 churn drivers"),
        Entry::new("board", "Board update sources"),
        Entry::new("digest", "Weekly digest draft"),
        Entry::new("onboarding", "Onboarding survey themes").archived(),
        Entry::new("pricing", "Pricing page copy").archived(),
    ]);
    let (selected, set_selected) = signal(Some("board".to_string()));
    let next = StoredValue::new(1_u32);

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 320px;">
            <AssistantThreadList
                items=items
                selected=selected
                on_select=move |id: String| set_selected.set(Some(id))
                on_new=move |()| {
                    let n = next.get_value();
                    next.set_value(n + 1);
                    let id = format!("new-{n}");
                    set_items.update(|v| v.insert(0, Entry::new(id.clone(), format!("New thread {n}"))));
                    set_selected.set(Some(id));
                }
                on_rename=move |(id, title): (String, String)| {
                    set_items.update(|v| v.iter_mut().filter(|t| t.id == id).for_each(|t| t.title.clone_from(&title)));
                }
                on_archive=move |(id, archived): (String, bool)| {
                    set_items.update(|v| v.iter_mut().filter(|t| t.id == id).for_each(|t| t.archived = archived));
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
    slug: "assistant-thread-list",
    name: "Assistant thread list",
    group: "Chat",
    summary: "The conversation sidebar for an assistant. A New thread button sits over an Active and an Archived section. Each row opens, renames in place, archives or restores, and deletes.",
    demo,
    leptos: include_str!("assistant_thread_list.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/assistant_thread_list.rs"),
    parts: &[
        &["AssistantThreadList", "root, new, section, heading, list, empty", "A nav. data-archived on the Archived section"],
        &["AssistantThreadListItem", "root, open, actions, action, rename", "A row. data-selected, data-archived. aria-current on the open button"],
    ],
    keys: &[
        &["Enter", "In the rename field: save the new title"],
        &["Escape", "In the rename field: leave the title as it was"],
    ],
    props: &[
        &["items", "Vec<ThreadListEntry>", "Conversations in display order. An entry has id, title and archived"],
        &["selected", "Option<String>", "Controlled open id. Pair with on_select"],
        &["default_selected", "String", "Open id when uncontrolled"],
        &["on_select", "Callback<String>", "Id of the opened row"],
        &["on_new", "Callback<()>", "New thread was pressed. You add the entry"],
        &["on_rename", "Callback<(String, String)>", "Id and trimmed new title"],
        &["on_archive", "Callback<(String, bool)>", "Id and whether it should now be archived"],
        &["on_delete", "Callback<String>", "Id to remove. You can confirm first"],
        &["AssistantThreadListItem", "entry, selected, on_*", "The row on its own, with on_select, on_rename(title), on_archive and on_delete"],
    ],
};
