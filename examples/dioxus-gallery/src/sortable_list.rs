use dioxus::prelude::*;
use hydroxy_dioxus::sortable_list::{Item, SortableList};

fn tasks() -> Vec<Item> {
    vec![
        Item::new("interviews", "Tag the Q3 interviews"),
        Item::new("tickets", "Import September tickets"),
        Item::new("digest", "Draft the weekly digest"),
        Item::new("sources", "Check sources on the board update"),
    ]
}

#[component]
pub fn SortableListDemo() -> Element {
    // Controlled: the owner stores the new order.
    let mut items = use_signal(tasks);

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 420px;",
            SortableList {
                label: "Priorities",
                items: Some(items()),
                on_reorder: move |next| items.set(next),
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);",
                {items.read().iter().map(|i| i.id.clone()).collect::<Vec<_>>().join(", ")}
            }
        }
    }
}
