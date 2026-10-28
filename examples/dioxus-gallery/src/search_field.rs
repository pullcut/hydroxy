use dioxus::prelude::*;
use hydroxy_dioxus::search_field::{SearchField, Size};

const SOURCES: [&str; 5] =
    ["interviews-q3.pdf", "tickets-sept.csv", "tickets-aug.csv", "survey-results.xlsx", "onboarding-notes.md"];

#[component]
pub fn SearchFieldDemo() -> Element {
    let mut query = use_signal(String::new);
    let q = query().to_lowercase();
    let matches: Vec<&str> = SOURCES.iter().filter(|s| s.contains(q.as_str())).copied().collect();

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            // Controlled: filters the list as you type. Esc or the cross clears it.
            div { style: "display: grid; gap: var(--space-4); max-width: 360px;",
                SearchField {
                    label: "Search sources",
                    placeholder: "Search sources",
                    value: Some(query()),
                    on_input: move |v| query.set(v),
                }
                p { style: "margin: 0; font: var(--text-label); letter-spacing: var(--tracking-label); text-transform: uppercase; color: var(--text-3);",
                    "{matches.len()} of {SOURCES.len()} sources"
                }
                ul { style: "margin: 0; padding: 0; list-style: none; display: grid; gap: var(--space-2); font: var(--text-sm);",
                    for s in matches {
                        li { key: "{s}", "{s}" }
                    }
                }
            }
            // Uncontrolled, large: for page-level search.
            SearchField { label: "Search the workspace", placeholder: "Search the workspace", size: Size::Lg }
        }
    }
}
