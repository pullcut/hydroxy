use dioxus::prelude::*;
use oxide_dioxus::hive_plot_chart::{HivePlotChart, Link, Node};

#[component]
pub fn HivePlotChartDemo() -> Element {
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

    rsx! {
        HivePlotChart {
            nodes,
            links,
            axes: tiers,
            value_label: "p95 latency",
            label: "Which parts of the system call which",
            format: |v: f64| format!("{v:.0} ms"),
        }
    }
}
