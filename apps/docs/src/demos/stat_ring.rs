use leptos::prelude::*;
use oxide_leptos::stat_ring::StatRing;

pub fn demo() -> AnyView {
    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-8);">
            <StatRing value=7_420.0 goal=10_000.0 label="Weekly active" />
            <StatRing value=38.0 goal=40.0 label="Story points" />
            <StatRing value=1.12 goal=1.0 label="Quota" format=|v: f64| format!("${v:.2}m") />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "stat-ring",
    name: "Stat ring",
    group: "Radial",
    summary: "A compact ring that fills towards a goal, with the share of the goal in the middle. Past the goal the ring stays full and the figure reads over 100%.",
    demo,
    leptos: include_str!("stat_ring.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/stat_ring.rs"),
    parts: &[
        &["StatRing", "root", "figure with a screen-reader table of value, goal and progress"],
        &["", "ring", "Focusable ring; focus or hover shows the reading"],
        &["", "track", "The full circle behind the value"],
        &["", "value", "The arc, filling clockwise from 12 o'clock"],
        &["", "center", "Share of the goal and the label"],
        &["", "tooltip", "Value and goal"],
    ],
    keys: &[&["Escape", "Hide the reading"]],
    props: &[
        &["value", "f64", "Required. The current value"],
        &["goal", "f64", "Required. What the ring fills towards"],
        &["label", "String", "Required. What it measures; shown in the middle"],
        &["size", "u16", "Diameter in px. Default 112"],
        &["thickness", "u16", "Ring width in px. Default 10"],
        &["format", "Fn(f64) -> String", "Formats the value and goal"],
    ],
};
