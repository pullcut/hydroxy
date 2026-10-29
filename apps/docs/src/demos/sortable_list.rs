use hydroxy_leptos::sortable_list::*;
use leptos::prelude::*;

fn tasks() -> Vec<Item> {
    vec![
        Item::new("interviews", "Tag the Q3 interviews"),
        Item::new("tickets", "Import September tickets"),
        Item::new("digest", "Draft the weekly digest"),
        Item::new("sources", "Check sources on the board update"),
    ]
}

pub fn demo() -> AnyView {
    // Controlled: the owner stores the new order.
    let (items, set_items) = signal(tasks());

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 420px;">
            <SortableList label="Priorities" items=items on_reorder=move |next| set_items.set(next) />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                {move || items.get().iter().map(|i| i.id.clone()).collect::<Vec<_>>().join(", ")}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "sortable-list",
    name: "Sortable list",
    group: "Data",
    summary: "A list you can reorder. Drag an item, or pick it up with Space and move it with the arrow keys. It shares its move logic with the Kanban board.",
    demo,
    leptos: include_str!("sortable_list.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/sortable_list.rs"),
    parts: &[
        &["SortableList", "root", "Wrapper. data-dragging while an item is moving"],
        &["SortableList", "list", "<ul role=list> drop zone. data-drop-end when a drag would land at the end"],
        &[
            "SortableList",
            "item",
            "<li role=listitem draggable tabindex=0>. data-grabbed while moving, data-drop-before when a drag would land above it",
        ],
        &["SortableList", "instructions", "Visually hidden keyboard help, linked from every item by aria-describedby"],
        &["SortableList", "live", "role=status region that announces pick-up, moves, drop and cancel"],
    ],
    keys: &[
        &["Space", "Picks up the focused item, or drops the one you're carrying"],
        &["ArrowUp / ArrowDown", "Moves the item up or down"],
        &["Escape", "Puts the item back where it started"],
        &["Tab", "Moves between items"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name for the list"],
        &["items", "Option<Vec<Item>>", "Controlled order. Pair with on_reorder"],
        &["default_items", "Vec<Item>", "Initial order when uncontrolled"],
        &["on_reorder", "Callback<Vec<Item>>", "Called with the full new order when an item lands somewhere new"],
        &["item", "Callback<Item, AnyView>", "Renders an item's content. Default: its title"],
        &["disabled", "bool", "Block every move"],
    ],
    events: &[],
};
