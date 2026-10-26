use leptos::prelude::*;
use oxide_leptos::treemap_chart::*;

fn team(name: &str, parts: &[(&str, f64)]) -> Tree {
    Tree::branch(name, parts.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

pub fn demo() -> AnyView {
    let spend = Tree::branch(
        "Spend",
        vec![
            team("Engineering", &[("Platform", 420.0), ("Product", 380.0), ("Data", 210.0), ("Security", 90.0)]),
            team("Sales", &[("Enterprise", 300.0), ("SMB", 180.0), ("Partners", 90.0)]),
            team("Marketing", &[("Brand", 150.0), ("Growth", 210.0), ("Events", 80.0)]),
            team("Support", &[("Tier 1", 140.0), ("Tier 2", 90.0)]),
            team("Operations", &[("People", 110.0), ("Finance", 70.0), ("Legal", 40.0)]),
        ],
    );

    view! { <TreemapChart root=spend label="Annual spend by team" /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "treemap-chart",
    name: "Treemap chart",
    group: "Charts",
    summary: "A tree as nested rectangles sized by value. Click a team to zoom in and use the breadcrumb to climb back out.",
    demo,
    leptos: include_str!("treemap_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/treemap_chart.rs"),
    parts: &[
        &["TreemapChart", "root", "figure labelled by `label`"],
        &["", "breadcrumb", "Path from the root to the current zoom; earlier steps are buttons"],
        &["", "cell", "Button for one node; `data-depth` 1 or 2, `data-leaf` when it can't zoom"],
        &["", "sr-table", "Every leaf with its path and value, for screen readers"],
    ],
    keys: &[
        &["Tab", "Moves between cells, then into the breadcrumb"],
        &["Enter / Space", "Zooms into the focused cell if it has children"],
    ],
    props: &[
        &["root", "Tree", "Required. Branch values come from their children"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px at an 800px drawing width. Default 380"],
        &["format", "Fn(f64) -> String", "Formats values in cells and the table"],
    ],
};
