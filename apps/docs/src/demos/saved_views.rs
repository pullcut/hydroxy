use std::collections::HashMap;

use hydroxy_leptos::saved_views::*;
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
struct Filter {
    status: &'static str,
    query: String,
}

const TICKETS: [(&str, &str); 6] = [
    ("Import September tickets", "open"),
    ("Tag the Q3 interviews", "open"),
    ("Draft the weekly digest", "closed"),
    ("Check sources on the board update", "closed"),
    ("Export interview notes", "open"),
    ("Archive 2025 notes", "closed"),
];

pub fn demo() -> AnyView {
    let filter = RwSignal::new(Filter { status: "open", query: String::new() });
    // The component keeps names and ids; the filters behind each id live here.
    let stored = StoredValue::new(HashMap::from([
        ("open".to_string(), Filter { status: "open", query: String::new() }),
        ("closed".to_string(), Filter { status: "closed", query: String::new() }),
    ]));

    let on_change = move |command: SavedViewsCommand| match command {
        SavedViewsCommand::Created(view) => stored.update_value(|m| {
            m.insert(view.id, filter.get_untracked());
        }),
        SavedViewsCommand::ActiveChange(Some(id)) => {
            if let Some(f) = stored.with_value(|m| m.get(&id).cloned()) {
                filter.set(f);
            }
        }
        SavedViewsCommand::Deleted(id) => stored.update_value(|m| {
            m.remove(&id);
        }),
        _ => {}
    };

    let shown = move || {
        let f = filter.get();
        TICKETS
            .iter()
            .filter(|(title, status)| {
                (f.status == "all" || *status == f.status) && title.to_lowercase().contains(&f.query.to_lowercase())
            })
            .map(|(title, status)| view! { <li>{*title} " · " {*status}</li> })
            .collect_view()
    };

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 520px;">
            <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);">
                <SavedViews
                    default_views=vec![SavedView::new("open", "Open tickets"), SavedView::new("closed", "Closed tickets")]
                    default_active="open"
                    default_view="open"
                    on_change=on_change
                />
                <select
                    aria-label="Status"
                    prop:value=move || filter.get().status
                    on:change=move |ev| {
                        let status = match event_target_value(&ev).as_str() {
                            "open" => "open",
                            "closed" => "closed",
                            _ => "all",
                        };
                        filter.update(|f| f.status = status);
                    }
                >
                    <option value="all">"All"</option>
                    <option value="open">"Open"</option>
                    <option value="closed">"Closed"</option>
                </select>
                <input
                    type="search"
                    aria-label="Search tickets"
                    placeholder="Search"
                    prop:value=move || filter.get().query
                    on:input=move |ev| filter.update(|f| f.query = event_target_value(&ev))
                />
            </div>
            <ul style="margin: 0; padding-left: var(--space-5); font: var(--text-sm); color: var(--text-2);">{shown}</ul>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "saved-views",
    name: "Saved views",
    group: "Navigation",
    summary: "A dropdown of saved filter presets. Pick one, save the current filters under a name, rename or delete the active one, or make it the view to open with. It keeps ids and names; you keep the filters.",
    demo,
    leptos: include_str!("saved_views.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/saved_views.rs"),
    parts: &[
        &["SavedViews", "root", "Wrapper"],
        &["Menu", "trigger, content, item", "The view list and actions, from the Menu component"],
        &["(inside)", "current", "The active view's name in the trigger, or \"Views\""],
        &[
            "(inside)",
            "check, name, default-mark",
            "In each view row: a check on the active one, its name, and a Default mark",
        ],
        &["(inside)", "separator, empty", "Divider before the actions; a note when there are no views"],
        &[
            "Dialog",
            "content, title, description, actions",
            "The save-as, rename and delete prompt, from the Dialog component",
        ],
        &["(inside)", "form", "<form>: Enter in the name field submits"],
    ],
    keys: &[
        &["ArrowDown / ArrowUp", "Moves through the views and actions in the open menu"],
        &["Enter / Space", "Picks the highlighted view or runs the action"],
        &["Escape", "Closes the menu, or the prompt"],
        &["Enter (in the name field)", "Saves or renames"],
    ],
    props: &[
        &["views", "Option<Vec<SavedView>>", "Controlled views. Pair with on_change and apply"],
        &["default_views", "Vec<SavedView>", "Initial views when uncontrolled. Each has an id and a name"],
        &["active", "Option<String>", "Controlled active view id"],
        &["default_active", "Option<String>", "Initially active view id when uncontrolled"],
        &["default_view", "Option<String>", "The view to open with, such as one loaded from storage"],
        &[
            "on_change",
            "Callback<SavedViewsCommand>",
            "Created(view), Renamed, Deleted, ActiveChange(Option<id>) and DefaultChange(Option<id>)",
        ],
    ],
};
