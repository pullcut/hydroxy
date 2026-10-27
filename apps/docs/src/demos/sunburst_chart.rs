use leptos::prelude::*;
use oxide_leptos::sunburst_chart::*;

fn team(name: &str, parts: &[(&str, f64)]) -> Tree {
    Tree::branch(name, parts.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

pub fn demo() -> AnyView {
    let spend = Tree::branch(
        "Spend",
        vec![
            Tree::branch(
                "Engineering",
                vec![
                    team("Platform", &[("Infra", 260.0), ("Tooling", 160.0)]),
                    team("Product", &[("Web", 220.0), ("Mobile", 160.0)]),
                    Tree::leaf("Data", 210.0),
                    Tree::leaf("Security", 90.0),
                ],
            ),
            team("Sales", &[("Enterprise", 300.0), ("SMB", 180.0), ("Partners", 90.0)]),
            team("Marketing", &[("Brand", 150.0), ("Growth", 210.0), ("Events", 80.0)]),
            team("Support", &[("Tier 1", 140.0), ("Tier 2", 90.0)]),
            team("Operations", &[("People", 110.0), ("Finance", 70.0), ("Legal", 40.0)]),
        ],
    );

    view! { <SunburstChart root=spend label="Annual spend by team" /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "sunburst-chart",
    name: "Sunburst chart",
    group: "Charts",
    summary: "A tree as concentric rings, each arc's sweep its share. Hover, touch or step to an arc to read it; click an arc, or press Enter, to zoom in; click the centre, press Backspace or use the breadcrumb to climb out.",
    demo,
    leptos: include_str!("sunburst_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/sunburst_chart.rs"),
    parts: &[
        &["SunburstChart", "root", "figure labelled by `label`; `data-active` while an arc is picked"],
        &["", "breadcrumb", "Path from the root to the current zoom; earlier steps are buttons"],
        &["", "plot", "Focusable group holding the rings, centre and tooltip; keeps focus while zooming"],
        &["", "arc", "Sector; `data-depth` is its ring, `data-leaf` when it can't zoom, `data-active` when picked"],
        &["", "center", "Shows the picked arc or the current level; zooms out when clicked"],
        &["", "tooltip", "Name, value and share of the current level for the picked arc"],
        &["", "legend", "The top ring's categories in colour order, when there are two or more"],
        &["", "sr-live", "Announces the picked arc and each zoom"],
        &["", "sr-table", "Every leaf with its path, value and share of the whole, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step to the next or previous arc, each parent before its children"],
        &["Home / End", "Jump to the first or last arc"],
        &["Enter / Space", "Zoom into the picked arc if it has parts"],
        &["Backspace", "Zoom out one level"],
        &["Escape", "Clear the picked arc; with none picked, zoom out one level"],
    ],
    props: &[
        &["root", "Tree", "Required. Branch values come from their children"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Diameter in px. Default 420"],
        &["depth", "u8", "Rings drawn, 1 to 4. Default 3"],
        &["format", "Fn(f64) -> String", "Formats values in the centre, tooltip and table"],
    ],
};
