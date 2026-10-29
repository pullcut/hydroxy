use hydroxy_leptos::icicle_chart::*;
use leptos::prelude::*;

fn module(name: &str, files: &[(&str, f64)]) -> Tree {
    Tree::branch(name, files.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

pub fn demo() -> AnyView {
    // A web app's JavaScript bundle, in kB after minification.
    let bundle = Tree::branch(
        "main.js",
        vec![
            Tree::branch(
                "node_modules",
                vec![
                    module("react-dom", &[("client", 118.0), ("server", 12.0)]),
                    module("date-fns", &[("locale", 46.0), ("format", 9.0), ("parse", 7.0)]),
                    module("chart.js", &[("controllers", 31.0), ("scales", 24.0), ("core", 38.0)]),
                    Tree::leaf("lodash", 24.0),
                ],
            ),
            Tree::branch(
                "src",
                vec![
                    module("pages", &[("dashboard", 28.0), ("settings", 14.0), ("billing", 11.0)]),
                    module("components", &[("table", 17.0), ("forms", 13.0), ("charts", 9.0)]),
                    Tree::leaf("api", 8.0),
                ],
            ),
            Tree::leaf("polyfills", 19.0),
        ],
    );

    view! { <IcicleChart root=bundle label="Bundle size by module" format=|v: f64| format!("{v:.0} kB") /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "icicle-chart",
    name: "Icicle chart",
    group: "Parts of a whole",
    summary: "A tree as rows of rectangles, one row per level, each as wide as its share of the row above. Click a cell to drill in; the top row or breadcrumb climbs back out.",
    demo,
    leptos: include_str!("icicle_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/icicle_chart.rs"),
    parts: &[
        &["IcicleChart", "root", "figure with a screen-reader table of every node"],
        &["", "breadcrumb", "Path from the root to the current zoom; earlier steps are buttons"],
        &["", "marks", "Focusable SVG; arrow keys move between cells"],
        &["", "cell", "One node's rectangle and label; `data-depth`, `data-leaf`, `data-active`"],
        &["", "tooltip", "Name, value and share of the zoomed node"],
        &["", "legend", "Top-level branches when there are two or more"],
    ],
    keys: &[
        &["Arrow keys", "Move between cells in tree order"],
        &["Home / End", "Jump to the first or last cell"],
        &["Enter / Space", "Drill into the focused cell, or out from the top row"],
        &["Escape", "Clear the focused cell, then climb out"],
    ],
    props: &[
        &["root", "Tree", "Required. Branch values come from their children"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px at an 800px drawing width. Default 320"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip and table"],
    ],
    events: crate::doc::NODE_EVENTS,
};
