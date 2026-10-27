use dioxus::prelude::*;
use oxide_dioxus::network_chart::{Link, NetworkChart, Node};

#[component]
pub fn NetworkChartDemo() -> Element {
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

    rsx! {
        NetworkChart { nodes, links, groups: teams, label: "Who works with whom" }
    }
}
