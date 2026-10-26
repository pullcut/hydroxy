use leptos::prelude::*;
use oxide_leptos::sankey_chart::*;

pub fn demo() -> AnyView {
    let flows = vec![
        Link::new("Organic", "Visits", 4200.0),
        Link::new("Paid", "Visits", 1800.0),
        Link::new("Referral", "Visits", 900.0),
        Link::new("Visits", "Signed up", 2100.0),
        Link::new("Visits", "Left", 4800.0),
        Link::new("Signed up", "Free", 1300.0),
        Link::new("Signed up", "Team", 600.0),
        Link::new("Signed up", "Business", 200.0),
        Link::new("Free", "Team", 150.0),
        Link::new("Free", "Churned", 700.0),
        Link::new("Team", "Retained", 640.0),
        Link::new("Team", "Churned", 110.0),
        Link::new("Business", "Retained", 190.0),
    ];

    view! { <SankeyChart links=flows label="Visits to plans" /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "sankey-chart",
    name: "Sankey chart",
    group: "Charts",
    summary: "Flows between stages, with link width by value. Hover or tab to a node or link to light its whole path, upstream and downstream.",
    demo,
    leptos: include_str!("sankey_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/sankey_chart.rs"),
    parts: &[
        &["SankeyChart", "root", "figure; `data-active` while a path is lit"],
        &["", "node", "Bar with its name and value; focusable, `data-highlighted` on the lit path"],
        &["", "link", "Ribbon whose width follows its value"],
        &["", "sr-table", "From, to and value for every link, for screen readers"],
    ],
    keys: &[&["Tab", "Moves between nodes and lights the focused node's path"]],
    props: &[
        &["links", "Vec<Link>", "Required. Flows by node name; nodes are created in order of first appearance"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["width", "u16", "Drawing width in px. Default 720"],
        &["height", "u16", "Drawing height in px. Default 340"],
        &["format", "Fn(f64) -> String", "Formats values in labels and the table"],
    ],
};
