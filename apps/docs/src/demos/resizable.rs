use hydroxy_leptos::resizable::{Orientation, Panel, ResizablePanels};
use leptos::prelude::*;

fn pane(title: &'static str, body: &'static str) -> impl IntoView {
    view! {
        <div style="padding: var(--space-5); display: grid; gap: var(--space-2);">
            <strong style="font: var(--text-ui);">{title}</strong>
            <span style="font: var(--text-sm); color: var(--text-3);">{body}</span>
        </div>
    }
}

pub fn demo() -> AnyView {
    let (sizes, set_sizes) = signal(vec![22.0, 48.0, 30.0]);
    let readout = move || sizes.get().iter().map(|s| format!("{s:.0}%")).collect::<Vec<_>>().join(" · ");

    view! {
        <div style="display: grid; gap: var(--space-8);">
            // Controlled, three panels. The first and last collapse.
            <div style="display: grid; gap: var(--space-3);">
                <div style="height: 240px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden; background: var(--surface-card);">
                    <ResizablePanels
                        panels=vec![
                            Panel::new(|| pane("Sources", "Collapses past half its minimum")).min(15.0).max(40.0).collapsible().label("sources"),
                            Panel::new(|| pane("Draft", "At least 30%")).min(30.0).label("draft"),
                            Panel::new(|| pane("Notes", "Enter on its separator toggles it")).min(20.0).max(45.0).collapsible().label("notes"),
                        ]
                        sizes=sizes
                        on_sizes_change=move |s| set_sizes.set(s)
                    />
                </div>
                <p style="margin: 0; font: var(--text-label); color: var(--text-3);">{readout}</p>
            </div>

            // Uncontrolled, stacked.
            <div style="height: 260px; border: 1px solid var(--border-1); border-radius: var(--radius-md); overflow: hidden; background: var(--surface-card);">
                <ResizablePanels
                    orientation=Orientation::Vertical
                    panels=vec![
                        Panel::new(|| pane("Query", "Top")).min(20.0).default_size(60.0).label("query"),
                        Panel::new(|| pane("Results", "Bottom")).min(20.0).label("results"),
                    ]
                />
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "resizable",
    name: "Resizable panels",
    group: "Layout",
    summary: "Panels in a row or column with separators you can drag or move with the keyboard. Per-panel minimum, maximum and collapse.",
    demo,
    leptos: include_str!("resizable.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/resizable.rs"),
    parts: &[
        &["ResizablePanels", "root", "Flex container. data-orientation. Fills its parent"],
        &["", "panel", "One panel, sized by flex weight. data-state=expanded|collapsed"],
        &[
            "",
            "separator",
            "role=separator, focusable. aria-valuenow is its panel's size in percent; data-dragging while dragged",
        ],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move a horizontal separator by one step"],
        &["ArrowUp / ArrowDown", "Move a vertical separator by one step"],
        &["Home", "Shrink the panel before to its minimum, or collapse it"],
        &["End", "Grow the panel before to its maximum"],
        &["Enter", "Collapse or expand the separator's panel"],
    ],
    props: &[
        &[
            "panels",
            "Vec<Panel>",
            "Required. Panel::new(content), then .min(), .max(), .default_size(), .collapsible(), .label()",
        ],
        &["orientation", "Orientation", "Horizontal (default) or Vertical"],
        &["sizes", "Vec<f64>", "Controlled sizes in percent. Pair with on_sizes_change"],
        &["on_sizes_change", "Callback<Vec<f64>>", "Called as sizes change"],
        &["step", "f64", "Keyboard step in percent. Default 5"],
    ],
};
