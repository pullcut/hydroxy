use std::collections::HashMap;

use dioxus::prelude::*;
use hydroxy_dioxus::saved_views::{SavedView, SavedViews, SavedViewsCommand};

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

#[component]
pub fn SavedViewsDemo() -> Element {
    let mut filter = use_signal(|| Filter { status: "open", query: String::new() });
    // The component keeps names and ids; the filters behind each id live here.
    let mut stored = use_signal(|| {
        HashMap::from([
            ("open".to_string(), Filter { status: "open", query: String::new() }),
            ("closed".to_string(), Filter { status: "closed", query: String::new() }),
        ])
    });

    let f = filter();
    let shown = TICKETS.iter().filter(|(title, status)| {
        (f.status == "all" || *status == f.status) && title.to_lowercase().contains(&f.query.to_lowercase())
    });

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 520px;",
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);",
                SavedViews {
                    default_views: vec![SavedView::new("open", "Open tickets"), SavedView::new("closed", "Closed tickets")],
                    default_active: "open",
                    default_view: "open",
                    on_change: move |command: SavedViewsCommand| match command {
                        SavedViewsCommand::Created(view) => {
                            stored.write().insert(view.id, filter());
                        }
                        SavedViewsCommand::ActiveChange(Some(id)) => {
                            let found = stored.read().get(&id).cloned();
                            if let Some(found) = found {
                                filter.set(found);
                            }
                        }
                        SavedViewsCommand::Deleted(id) => {
                            stored.write().remove(&id);
                        }
                        _ => {}
                    },
                }
                select {
                    "aria-label": "Status",
                    value: f.status,
                    onchange: move |e| {
                        let status = match e.value().as_str() {
                            "open" => "open",
                            "closed" => "closed",
                            _ => "all",
                        };
                        filter.write().status = status;
                    },
                    option { value: "all", "All" }
                    option { value: "open", "Open" }
                    option { value: "closed", "Closed" }
                }
                input {
                    r#type: "search",
                    "aria-label": "Search tickets",
                    placeholder: "Search",
                    value: "{f.query}",
                    oninput: move |e| filter.write().query = e.value(),
                }
            }
            ul { style: "margin: 0; padding-left: var(--space-5); font: var(--text-sm); color: var(--text-2);",
                for (title , status) in shown {
                    li { key: "{title}", "{title} · {status}" }
                }
            }
        }
    }
}
