use dioxus::prelude::*;
use hydroxy_dioxus::assistant_thread_list::{AssistantThreadList, Entry};

#[component]
pub fn AssistantThreadListDemo() -> Element {
    let mut items = use_signal(|| {
        vec![
            Entry::new("churn", "Q3 churn drivers"),
            Entry::new("board", "Board update sources"),
            Entry::new("digest", "Weekly digest draft"),
            Entry::new("onboarding", "Onboarding survey themes").archived(),
            Entry::new("pricing", "Pricing page copy").archived(),
        ]
    });
    let mut selected = use_signal(|| Some("board".to_string()));
    let mut next = use_signal(|| 1_u32);

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 320px;",
            AssistantThreadList {
                items,
                selected,
                on_select: move |id: String| selected.set(Some(id)),
                on_new: move |()| {
                    let n = *next.peek();
                    next += 1;
                    let id = format!("new-{n}");
                    items.write().insert(0, Entry::new(id.clone(), format!("New thread {n}")));
                    selected.set(Some(id));
                },
                on_rename: move |(id, title): (String, String)| {
                    items.write().iter_mut().filter(|t| t.id == id).for_each(|t| t.title.clone_from(&title));
                },
                on_archive: move |(id, archived): (String, bool)| {
                    items.write().iter_mut().filter(|t| t.id == id).for_each(|t| t.archived = archived);
                },
                on_delete: move |id: String| items.write().retain(|t| t.id != id),
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                match selected() {
                    Some(id) => format!("Open: {id}"),
                    None => "Nothing open.".to_string(),
                }
            }
        }
    }
}
