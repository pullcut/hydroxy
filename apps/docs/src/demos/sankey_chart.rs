use hydroxy_leptos::sankey_chart::*;
use leptos::prelude::*;

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
    group: "Relationships",
    summary: "Flows between stages, with link width by value. Hover, touch or step to a node or link with the arrow keys to light its whole path, upstream and downstream, and read its flows.",
    demo,
    leptos: include_str!("sankey_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/sankey_chart.rs"),
    parts: &[
        &["SankeyChart", "root", "figure; `data-active` while a path is lit"],
        &["", "plot", "Focusable group holding the drawing and the tooltip"],
        &["", "node", "Bar with its name and value; `data-highlighted` on the lit path, `data-selected` when picked"],
        &["", "link", "Ribbon whose width follows its value; `data-highlighted` on the lit path"],
        &["", "tooltip", "A node's inflow and outflow, or a link's source → target and value"],
        &["", "sr-live", "Announces the picked node or link"],
        &["", "sr-table", "From, to and value for every link, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step through the nodes, then the links"],
        &["Home / End", "Jump to the first node or the last link"],
        &["Escape", "Clear the lit path"],
    ],
    props: &[
        &["links", "Vec<Link>", "Required. Flows by node name; nodes are created in order of first appearance"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["width", "u16", "Drawing width in px. Default 720"],
        &["height", "u16", "Drawing height in px. Default 340"],
        &["format", "Fn(f64) -> String", "Formats values in labels, the tooltip and the table"],
    ],
};
