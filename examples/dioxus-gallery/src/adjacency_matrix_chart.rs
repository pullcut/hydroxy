use dioxus::prelude::*;
use oxide_dioxus::adjacency_matrix_chart::{AdjacencyMatrixChart, Link, MatrixOrder, Node};

#[component]
pub fn AdjacencyMatrixChartDemo() -> Element {
    let people = [("Ana", 0), ("Bo", 0), ("Cy", 0), ("Dee", 1), ("Eko", 1), ("Fin", 1), ("Gia", 2), ("Hal", 2), ("Ines", 2), ("Jun", 0)];
    let nodes: Vec<Node> = people.iter().map(|(n, g)| Node::new(*n, *g)).collect();
    // Messages exchanged last week between people on three teams.
    let pairs = [
        ("Ana", "Bo", 34.0),
        ("Ana", "Cy", 21.0),
        ("Bo", "Cy", 18.0),
        ("Ana", "Jun", 12.0),
        ("Cy", "Jun", 9.0),
        ("Dee", "Eko", 27.0),
        ("Dee", "Fin", 15.0),
        ("Eko", "Fin", 22.0),
        ("Gia", "Hal", 30.0),
        ("Gia", "Ines", 16.0),
        ("Hal", "Ines", 11.0),
        ("Bo", "Dee", 4.0),
        ("Fin", "Gia", 6.0),
        ("Jun", "Ines", 3.0),
    ];
    let links: Vec<Link> = pairs.iter().map(|(a, b, v)| Link::new(*a, *b, *v)).collect();

    rsx! {
        AdjacencyMatrixChart { nodes, links, label: "Messages between people", order: MatrixOrder::Cluster }
    }
}
