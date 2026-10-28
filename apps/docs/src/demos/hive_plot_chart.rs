use leptos::prelude::*;
use oxide_leptos::hive_plot_chart::*;

pub fn demo() -> AnyView {
    // p95 latency in ms places each part of the system along its tier's axis.
    let parts = [
        ("Web app", 0, 120.0),
        ("iOS app", 0, 180.0),
        ("Android app", 0, 210.0),
        ("Partner API", 0, 95.0),
        ("Auth", 1, 18.0),
        ("Catalog", 1, 42.0),
        ("Search", 1, 85.0),
        ("Cart", 1, 30.0),
        ("Checkout", 1, 140.0),
        ("Payments", 1, 260.0),
        ("Notifications", 1, 55.0),
        ("Postgres", 2, 12.0),
        ("Redis", 2, 2.0),
        ("Elasticsearch", 2, 38.0),
        ("S3", 2, 65.0),
        ("Kafka", 2, 8.0),
    ];
    let nodes: Vec<Node> = parts.iter().map(|(id, axis, ms)| Node::new(*id, *axis, *ms)).collect();
    let calls = [
        ("Web app", "Auth"),
        ("Web app", "Catalog"),
        ("Web app", "Search"),
        ("Web app", "Cart"),
        ("Web app", "Checkout"),
        ("Web app", "S3"),
        ("iOS app", "Auth"),
        ("iOS app", "Catalog"),
        ("iOS app", "Cart"),
        ("iOS app", "Checkout"),
        ("Android app", "Auth"),
        ("Android app", "Search"),
        ("Android app", "Checkout"),
        ("Partner API", "Auth"),
        ("Partner API", "Catalog"),
        ("Auth", "Postgres"),
        ("Auth", "Redis"),
        ("Catalog", "Postgres"),
        ("Catalog", "Redis"),
        ("Catalog", "S3"),
        ("Search", "Elasticsearch"),
        ("Cart", "Redis"),
        ("Checkout", "Postgres"),
        ("Checkout", "Kafka"),
        ("Payments", "Postgres"),
        ("Payments", "Kafka"),
        ("Notifications", "Kafka"),
    ];
    let links: Vec<Link> = calls.iter().map(|(a, b)| Link::new(*a, *b, 1.0)).collect();
    let tiers = vec!["Clients".to_string(), "Services".to_string(), "Stores".to_string()];

    view! {
        <HivePlotChart
            nodes=nodes
            links=links
            axes=tiers
            value_label="p95 latency"
            label="Which parts of the system call which"
            format=|v: f64| format!("{v:.0} ms")
        />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "hive-plot-chart",
    name: "Hive plot",
    group: "Relationships",
    summary: "Two to four axes from the centre, one per kind of node. Each node sits on its kind's axis as far out as its value, and links curve between axes, so the same data always draws the same picture. Hover, touch or step to a node to light its links.",
    demo,
    leptos: include_str!("hive_plot_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/hive_plot_chart.rs"),
    parts: &[
        &["HivePlotChart", "root", "figure; `data-active` while a node is picked out"],
        &["", "plot", "Focusable group holding the drawing and the tooltip"],
        &["", "axis", "One line per kind, centre outwards, named at its tip by `axis-label`"],
        &["", "link", "Curve between two axes in the source axis's colour; `data-highlighted` when it touches the picked node"],
        &["", "node", "Dot on its axis; `data-highlighted` on the picked node and its neighbours, `data-selected` on the picked node"],
        &["", "tooltip", "Kind, value and link count of the picked node"],
        &["", "scale-note", "The value range from centre to tip, shared by every axis"],
        &["", "sr-live", "Announces the picked node"],
        &["", "sr-table", "Every node axis by axis with its value, links and neighbours"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step to the next or previous node, axis by axis, centre outwards"],
        &["Home / End", "Jump to the first or last node"],
        &["Escape", "Clear the picked node"],
    ],
    props: &[
        &["nodes", "Vec<Node>", "Required. Node::new(id, axis, value)"],
        &["links", "Vec<Link>", "Required. Link::new(a, b, value) by id; links within one axis are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["axes", "Vec<String>", "Axis names; two to four, and their count sets the number of axes. Default: one per axis used"],
        &["value_label", "String", "Name of the value along the axes. Default \"Value\""],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip, scale note and table"],
    ],
};
