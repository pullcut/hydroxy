use dioxus::prelude::*;
use oxide_dioxus::dependency_graph_chart::{DependencyGraphChart, Link, Node};

#[component]
pub fn DependencyGraphChartDemo() -> Element {
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

    rsx! {
        DependencyGraphChart { nodes, links, group_names, label: "Package dependencies" }
    }
}
