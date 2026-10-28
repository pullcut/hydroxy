use dioxus::prelude::*;
use hydroxy_dioxus::thread_list::{ThreadList, ThreadSummary};

#[component]
pub fn ThreadListDemo() -> Element {
    let mut items = use_signal(|| {
        vec![
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
        ]
    });
    let mut selected = use_signal(|| Some("board".to_string()));

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 360px;",
            ThreadList {
                items,
                selected,
                on_select: move |id: String| {
                    // Opening a conversation reads it.
                    items.write().iter_mut().filter(|t| t.id == id).for_each(|t| t.unread = false);
                    selected.set(Some(id));
                },
                on_rename: move |(id, title): (String, String)| {
                    items.write().iter_mut().filter(|t| t.id == id).for_each(|t| t.title.clone_from(&title));
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
