use dioxus::prelude::*;
use oxide_dioxus::button::{Button, ButtonVariant};
use oxide_dioxus::tour::{Side, Tour, TourStep};

#[component]
pub fn TourDemo() -> Element {
    let mut open = use_signal(|| false);
    let mut outcome = use_signal(String::new);
    let steps = vec![
        TourStep::centred("Welcome to Projects", "A quick look at where things live. It takes about thirty seconds."),
        TourStep::new("tour-demo-search", "Find anything", "Search across projects, notes and files. Press / from anywhere to jump here."),
        TourStep::new("tour-demo-new", "Start a project", "New projects begin from a template, so research plans come with interview guides ready to go.").side(Side::Start),
        TourStep::new("tour-demo-list", "Recent work", "Your last five projects, newest first. Drag one onto the sidebar to pin it.").side(Side::End),
        TourStep::new("#tour-demo-missing", "Shared with you", "When teammates share a project it shows up here. Nothing yet, so this step sits in the middle."),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-4);",
            div { style: "display: flex; gap: var(--space-3); align-items: center;",
                Button { on_click: move |_| open.set(true), "Take the tour" }
                span { style: "font: var(--text-xs); color: var(--text-3);", "{outcome}" }
            }
            div { style: "display: grid; gap: var(--space-4); padding: var(--space-5); border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-card);",
                div { style: "display: flex; gap: var(--space-3);",
                    div { id: "tour-demo-search", "data-scope": "input", "data-part": "field", style: "flex: 1; min-width: 0;",
                        input { r#type: "search", "aria-label": "Search projects", placeholder: "Search projects",
                            style: "flex: 1; min-width: 0; border: none; background: transparent; font: var(--text-body); color: inherit; outline: none;" }
                    }
                    span { id: "tour-demo-new", Button { variant: ButtonVariant::Accent, "New project" } }
                }
                ul { id: "tour-demo-list", style: "margin: 0; padding: 0; list-style: none; display: grid; gap: var(--space-2); max-width: 320px; font: var(--text-sm);",
                    li { "Q3 onboarding interviews" }
                    li { "Pricing page experiment" }
                    li { "Churn survey, September" }
                }
            }
            Tour {
                steps,
                open: Some(open()),
                on_open_change: move |o| open.set(o),
                on_end: move |end| outcome.set(format!("Ended: {end:?}")),
            }
        }
    }
}
