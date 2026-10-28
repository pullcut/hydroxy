use dioxus::prelude::*;
use hydroxy_dioxus::edge_bundling_chart::{EdgeBundlingChart, Link, Tree};

fn package(name: &str, modules: &[&str]) -> Tree {
    Tree::branch(name, modules.iter().map(|m| Tree::leaf(*m, 1.0)).collect())
}

#[component]
pub fn EdgeBundlingChartDemo() -> Element {
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

    rsx! {
        EdgeBundlingChart { tree, links, label: "Module imports", out_label: "Imports", in_label: "Imported by" }
    }
}
