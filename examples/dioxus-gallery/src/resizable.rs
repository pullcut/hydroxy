use dioxus::prelude::*;
use oxide_dioxus::resizable::{Orientation, Panel, ResizablePanels};

fn pane(title: &'static str, body: &'static str) -> Element {
    rsx! {
        div { style: "padding: var(--space-5); display: grid; gap: var(--space-2);",
            strong { style: "font: var(--text-ui);", {title} }
            span { style: "font: var(--text-sm); color: var(--text-3);", {body} }
        }
    }
}

#[component]
pub fn ResizableDemo() -> Element {
    let mut sizes = use_signal(|| vec![22.0, 48.0, 30.0]);
    let readout = sizes.read().iter().map(|s| format!("{s:.0}%")).collect::<Vec<_>>().join(" · ");

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            div { style: "display: grid; gap: var(--space-3);",
                div { style: "height: 240px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden; background: var(--surface-card);",
                    ResizablePanels {
                        panels: vec![
                            Panel::new(pane("Sources", "Collapses past half its minimum")).min(15.0).max(40.0).collapsible().label("sources"),
                            Panel::new(pane("Draft", "At least 30%")).min(30.0).label("draft"),
                            Panel::new(pane("Notes", "Enter on its separator toggles it")).min(20.0).max(45.0).collapsible().label("notes"),
                        ],
                        sizes: Some(sizes()),
                        on_sizes_change: move |s| sizes.set(s),
                    }
                }
                p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", {readout} }
            }
            div { style: "height: 260px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden; background: var(--surface-card);",
                ResizablePanels {
                    orientation: Orientation::Vertical,
                    panels: vec![
                        Panel::new(pane("Query", "Top")).min(20.0).default_size(60.0).label("query"),
                        Panel::new(pane("Results", "Bottom")).min(20.0).label("results"),
                    ],
                }
            }
        }
    }
}
