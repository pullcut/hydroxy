use hydroxy_leptos::adjacency_matrix_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let people = [
        ("Ana", 0),
        ("Bo", 0),
        ("Cy", 0),
        ("Dee", 1),
        ("Eko", 1),
        ("Fin", 1),
        ("Gia", 2),
        ("Hal", 2),
        ("Ines", 2),
        ("Jun", 0),
    ];
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

    view! { <AdjacencyMatrixChart nodes=nodes links=links label="Messages between people" order=MatrixOrder::Cluster /> }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "adjacency-matrix-chart",
    name: "Adjacency matrix chart",
    group: "Relationships",
    summary: "Nodes against nodes, each cell shaded by the link between them. Order by name, by degree, or by cluster so that tightly linked groups form blocks along the diagonal.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("adjacency_matrix_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/adjacency_matrix_chart.rs"),
    parts: &[
        &["AdjacencyMatrixChart", "root", "figure"],
        &["", "row-label / col-label", "Node names; `data-active` on the active cell's row and column"],
        &["", "cell", "Shaded by `--level`; `data-empty` with no link, `data-cross` on the active row and column"],
        &["", "tooltip", "Source, target and value of the active cell"],
        &["", "sr-table", "Every link as source, target and value"],
    ],
    keys: &[
        &["Arrow keys", "Move between cells"],
        &["Home / End", "First or last cell in the row"],
        &["Escape", "Clear the active cell"],
    ],
    props: &[
        &["nodes", "Vec<Node>", "Required. Unique ids, which are also the labels"],
        &["links", "Vec<Link>", "Required. Links by node id; repeats add up, unknown ids are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["order", "MatrixOrder", "Name (default), Degree or Cluster"],
        &["directed", "bool", "Links run one way. Default false, which mirrors each link"],
        &["cell_size", "u16", "Cell size in px. Default 28"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip, readout and table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
