use hydroxy_leptos::edge_bundling_chart::*;
use leptos::prelude::*;

fn package(name: &str, modules: &[&str]) -> Tree {
    Tree::branch(name, modules.iter().map(|m| Tree::leaf(*m, 1.0)).collect())
}

#[component]
pub fn Demo() -> impl IntoView {
    let tree = Tree::branch(
        "app",
        vec![
            package("ui", &["Button", "Dialog", "Form", "Table", "Chart", "Theme"]),
            package("state", &["Store", "Router", "Session", "Cache"]),
            package("data", &["Client", "Schema", "Query", "Sync"]),
            package("platform", &["Logger", "Metrics", "Config", "Auth"]),
        ],
    );
    // Which module imports which.
    let imports = [
        ("Dialog", "Button"),
        ("Form", "Button"),
        ("Form", "Schema"),
        ("Form", "Store"),
        ("Table", "Query"),
        ("Table", "Theme"),
        ("Table", "Store"),
        ("Chart", "Theme"),
        ("Chart", "Query"),
        ("Dialog", "Theme"),
        ("Button", "Theme"),
        ("Store", "Cache"),
        ("Store", "Sync"),
        ("Router", "Session"),
        ("Router", "Metrics"),
        ("Session", "Auth"),
        ("Session", "Store"),
        ("Cache", "Logger"),
        ("Client", "Auth"),
        ("Client", "Config"),
        ("Client", "Logger"),
        ("Client", "Metrics"),
        ("Query", "Client"),
        ("Query", "Cache"),
        ("Query", "Schema"),
        ("Sync", "Client"),
        ("Sync", "Store"),
        ("Schema", "Logger"),
        ("Auth", "Config"),
        ("Auth", "Logger"),
        ("Metrics", "Config"),
        ("Logger", "Config"),
    ];
    let links: Vec<Link> = imports.iter().map(|(a, b)| Link::new(*a, *b, 1.0)).collect();

    view! { <EdgeBundlingChart tree=tree links=links label="Module imports" out_label="Imports" in_label="Imported by" /> }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "edge-bundling-chart",
    name: "Edge bundling",
    group: "Relationships",
    summary: "Hierarchical edge bundling: the leaves of a tree sit round a circle and each link follows the tree between them, so links between the same branches gather into bundles. Hover, touch or step to a leaf to see what it links to in one colour and what links to it in another.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("edge_bundling_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/edge_bundling_chart.rs"),
    parts: &[
        &["EdgeBundlingChart", "root", "figure; `data-active` while a leaf is picked out"],
        &["", "plot", "Focusable group holding the drawing and the tooltip"],
        &["", "link", "B-spline routed through the tree; `data-role` is `out` or `in` relative to the picked leaf"],
        &[
            "",
            "leaf",
            "Dot and rotated label on the ring; `data-role` is `target`, `source` or `both` relative to the picked leaf, `data-selected` on the picked leaf",
        ],
        &["", "tooltip", "Group and the number of links each way"],
        &["", "legend", "The two link colours"],
        &["", "sr-live", "Announces the picked leaf"],
        &["", "sr-table", "Every leaf with its group and what it links to and from"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step clockwise or anticlockwise round the circle"],
        &["Home / End", "Jump to the first or last leaf"],
        &["Escape", "Clear the picked leaf"],
    ],
    props: &[
        &[
            "tree",
            "Tree",
            "Required. Tree::branch(name, children) and Tree::leaf(name, value); leaves go round the circle in order",
        ],
        &["links", "Vec<Link>", "Required. Link::new(from, to, value) by leaf name; values are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["beta", "f64", "Bundling strength from 0 (straight) to 1 (follows the tree). Default 0.85"],
        &["out_label / in_label", "String", "What a link means each way. Default \"Links to\" and \"Linked from\""],
        &["size", "u16", "Drawing width and height in px. Default 600"],
    ],
    events: crate::doc::CHART_EVENTS,
};
