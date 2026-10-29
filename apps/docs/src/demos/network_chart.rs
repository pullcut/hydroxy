use hydroxy_leptos::network_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let people = [
        ("Ada", 0),
        ("Ben", 0),
        ("Cleo", 0),
        ("Dev", 0),
        ("Eli", 1),
        ("Fay", 1),
        ("Gus", 1),
        ("Hana", 1),
        ("Ivo", 2),
        ("Jo", 2),
        ("Kit", 2),
        ("Lena", 2),
    ];
    let nodes: Vec<Node> = people.iter().map(|(n, g)| Node::new(*n, *g)).collect();
    let pairs = [
        ("Ada", "Ben", 3.0),
        ("Ada", "Cleo", 2.0),
        ("Ben", "Cleo", 2.0),
        ("Cleo", "Dev", 1.0),
        ("Ada", "Dev", 1.0),
        ("Eli", "Fay", 3.0),
        ("Fay", "Gus", 2.0),
        ("Gus", "Hana", 2.0),
        ("Eli", "Hana", 1.0),
        ("Eli", "Gus", 1.0),
        ("Ivo", "Jo", 3.0),
        ("Jo", "Kit", 2.0),
        ("Kit", "Lena", 2.0),
        ("Ivo", "Lena", 1.0),
        ("Dev", "Eli", 1.0),
        ("Hana", "Ivo", 1.0),
        ("Cleo", "Kit", 1.0),
    ];
    let links: Vec<Link> = pairs.iter().map(|(a, b, v)| Link::new(*a, *b, *v)).collect();

    let teams = vec!["Design".to_string(), "Platform".to_string(), "Growth".to_string()];

    view! { <NetworkChart nodes=nodes links=links groups=teams label="Who works with whom" /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "network-chart",
    name: "Network chart",
    group: "Relationships",
    summary: "Nodes and links in a force-directed layout. The layout is a fixed number of seeded steps, so it is the same on every render. Hover, touch or step to a node with the arrow keys to pick out its neighbours and read its group, links and weight.",
    demo,
    leptos: include_str!("network_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/network_chart.rs"),
    parts: &[
        &["NetworkChart", "root", "figure; `data-active` while a node is picked out"],
        &["", "plot", "Focusable group holding the drawing and the tooltip"],
        &[
            "",
            "node",
            "Circle sized by its links, with a label; `data-highlighted` on the picked node and its neighbours, `data-selected` on the picked node",
        ],
        &["", "link", "Line whose width follows its value; `data-highlighted` when it touches the picked node"],
        &["", "tooltip", "Name, group, link count and total link weight of the picked node"],
        &["", "legend", "One swatch per group, when there are two or more"],
        &["", "sr-live", "Announces the picked node"],
        &["", "sr-table", "Every node with its group, links, weight and neighbours, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step to the next or previous node"],
        &["Home / End", "Jump to the first or last node"],
        &["Escape", "Clear the picked node"],
    ],
    props: &[
        &["nodes", "Vec<Node>", "Required. Unique ids and a group number that sets the colour"],
        &["links", "Vec<Link>", "Required. Links by node id; unknown ids are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["groups", "Vec<String>", "Group names by group number. Default: \"Group 1\", \"Group 2\"…"],
        &["format", "Fn(f64) -> String", "Formats link weights in the tooltip and table"],
        &["width", "u16", "Drawing width in px. Default 640"],
        &["height", "u16", "Drawing height in px. Default 420"],
        &["iterations", "u16", "Layout steps. Default 300"],
        &["seed", "u32", "Seed for the starting positions. Default 1"],
    ],
    events: crate::doc::CHART_EVENTS,
};
