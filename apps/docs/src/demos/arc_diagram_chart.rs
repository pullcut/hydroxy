use hydroxy_leptos::arc_diagram_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let people = [
        ("Amara", 0),
        ("Bilal", 0),
        ("Chen", 0),
        ("Dana", 0),
        ("Elif", 1),
        ("Farid", 1),
        ("Grace", 1),
        ("Hugo", 1),
        ("Ines", 1),
        ("Jonas", 2),
        ("Kemi", 2),
        ("Luca", 2),
        ("Mei", 2),
    ];
    let nodes: Vec<Node> = people.iter().map(|(n, g)| Node::new(*n, *g)).collect();
    // Papers co-authored over the last three years.
    let papers = [
        ("Amara", "Bilal", 6.0),
        ("Amara", "Chen", 4.0),
        ("Bilal", "Chen", 3.0),
        ("Chen", "Dana", 2.0),
        ("Amara", "Dana", 1.0),
        ("Elif", "Farid", 5.0),
        ("Elif", "Grace", 3.0),
        ("Farid", "Grace", 4.0),
        ("Grace", "Hugo", 2.0),
        ("Hugo", "Ines", 3.0),
        ("Elif", "Ines", 1.0),
        ("Farid", "Hugo", 2.0),
        ("Jonas", "Kemi", 5.0),
        ("Kemi", "Luca", 3.0),
        ("Luca", "Mei", 4.0),
        ("Jonas", "Mei", 2.0),
        ("Chen", "Grace", 2.0),
        ("Dana", "Jonas", 1.0),
        ("Bilal", "Luca", 1.0),
        ("Ines", "Kemi", 2.0),
        ("Amara", "Elif", 1.0),
    ];
    let links: Vec<Link> = papers.iter().map(|(a, b, v)| Link::new(*a, *b, *v)).collect();
    let teams = vec!["Vision".to_string(), "Language".to_string(), "Robotics".to_string()];

    view! {
        <ArcDiagramChart
            nodes=nodes
            links=links
            groups=teams
            label="Papers co-authored across the lab"
            format=|v: f64| format!("{v:.0} papers")
        />
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "arc-diagram-chart",
    name: "Arc diagram",
    group: "Relationships",
    summary: "Nodes in a row with each link drawn as an arc above them. Ordered by group, a team's own arcs stay short and the links between teams stand out as the long ones. Hover, touch or step to a node to light its arcs.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("arc_diagram_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/arc_diagram_chart.rs"),
    parts: &[
        &["ArcDiagramChart", "root", "figure; `data-active` while a node is picked out"],
        &["", "plot", "Focusable group holding the drawing and the tooltip"],
        &["", "baseline", "The line the nodes sit on"],
        &["", "arc", "Semicircle per link, as wide as its value; `data-highlighted` when it touches the picked node"],
        &[
            "",
            "node",
            "Dot sized by its links, with a slanted label; `data-highlighted` on the picked node and its neighbours, `data-selected` on the picked node",
        ],
        &["", "tooltip", "Group, link count, total weight and who it links to"],
        &["", "legend", "One swatch per group, when there are two or more"],
        &["", "sr-live", "Announces the picked node"],
        &["", "sr-table", "Every node left to right with its group, links, weight and neighbours"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step to the next or previous node along the line"],
        &["Home / End", "Jump to the first or last node"],
        &["Escape", "Clear the picked node"],
    ],
    props: &[
        &["nodes", "Vec<Node>", "Required. Node::new(id, group); ids are unique and are the labels"],
        &["links", "Vec<Link>", "Required. Link::new(a, b, value); unknown ids and self-links are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["groups", "Vec<String>", "Group names by group number. Default: \"Group 1\", \"Group 2\"…"],
        &["order", "Order", "Group (default: by group, most-linked first), Degree or Input"],
        &["weighted", "bool", "Arc width follows link value. Default true"],
        &["format", "Fn(f64) -> String", "Formats link values in the tooltip and table"],
        &["width", "u16", "Drawing width in px. Default 720"],
    ],
    events: crate::doc::CHART_EVENTS,
};
