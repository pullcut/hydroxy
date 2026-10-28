use dioxus::prelude::*;
use oxide_dioxus::arc_diagram_chart::{ArcDiagramChart, Link, Node};

#[component]
pub fn ArcDiagramChartDemo() -> Element {
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

    rsx! {
        ArcDiagramChart {
            nodes,
            links,
            groups: teams,
            label: "Papers co-authored across the lab",
            format: |v: f64| format!("{v:.0} papers"),
        }
    }
}
