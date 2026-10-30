use hydroxy_leptos::dependency_graph_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // Packages in a monorepo: 0 libraries, 1 services, 2 apps. A link means "is used by".
    let packages = [
        ("config", 0),
        ("logging", 0),
        ("db", 0),
        ("auth", 1),
        ("billing", 1),
        ("search", 1),
        ("api", 1),
        ("worker", 1),
        ("web", 2),
        ("admin", 2),
        ("mobile", 2),
    ];
    let nodes: Vec<Node> = packages.iter().map(|(n, g)| Node::new(*n, *g)).collect();
    let uses = [
        ("config", "logging"),
        ("config", "db"),
        ("logging", "db"),
        ("db", "auth"),
        ("db", "billing"),
        ("db", "search"),
        ("auth", "api"),
        ("billing", "api"),
        ("search", "api"),
        ("billing", "worker"),
        ("logging", "worker"),
        ("api", "web"),
        ("api", "admin"),
        ("api", "mobile"),
        ("auth", "admin"),
    ];
    let links: Vec<Link> = uses.iter().map(|(a, b)| Link::new(*a, *b, 1.0)).collect();
    let group_names: Vec<String> = ["Library", "Service", "App"].iter().map(|s| (*s).to_string()).collect();

    view! { <DependencyGraphChart nodes=nodes links=links group_names=group_names label="Package dependencies" /> }
        
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "dependency-graph-chart",
    name: "Dependency graph chart",
    group: "Relationships",
    summary: "A directed graph in layers, read left to right. Cycles are broken and drawn dashed, long links route between nodes, and crossings are kept low. Hover or focus a node to trace everything upstream and downstream.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("dependency_graph_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/dependency_graph_chart.rs"),
    parts: &[
        &["DependencyGraphChart", "root", "figure; `data-active` while a node is traced"],
        &[
            "",
            "node",
            "Box for one node, coloured by group; `data-lit` on its upstream and downstream, `data-dim` on the rest",
        ],
        &["", "link", "Curve from source to target; `data-reversed` when it closes a cycle"],
        &["", "tooltip", "Layer, upstream and downstream counts, and flow in and out"],
        &["", "sr-table", "Every node with its layer, inputs and outputs"],
    ],
    keys: &[
        &["Arrow keys", "Step through nodes, column by column"],
        &["Home / End", "First or last node"],
        &["Escape", "Clear the active node"],
    ],
    props: &[
        &["nodes", "Vec<Node>", "Required. Unique ids and a group number that sets the colour"],
        &["links", "Vec<Link>", "Required. Source feeds target; unknown ids and self-links are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["group_names", "Vec<String>", "Legend names by group number"],
        &["width", "u16", "Drawing width in px. Default 720"],
        &["height", "u16", "Drawing height in px. Default 360"],
        &["node_width", "u16", "Node box width in px. Default 112"],
        &["format", "Fn(f64) -> String", "Formats flow in and out"],
    ],
    events: crate::doc::CHART_EVENTS,
};
