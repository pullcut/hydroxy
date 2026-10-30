use hydroxy_leptos::bubble_pack_chart::*;
use leptos::prelude::*;

fn folder(name: &str, files: &[(&str, f64)]) -> Tree {
    Tree::branch(name, files.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

#[component]
pub fn Demo() -> impl IntoView {
    // Disk usage of a monorepo checkout, in MB.
    let repo = Tree::branch(
        "repo",
        vec![
            Tree::branch(
                "apps",
                vec![
                    folder("web", &[("node_modules", 412.0), ("dist", 38.0), ("src", 9.0)]),
                    folder("docs", &[("node_modules", 188.0), ("public", 64.0), ("content", 12.0)]),
                    folder("mobile", &[("ios", 96.0), ("android", 121.0), ("src", 14.0)]),
                ],
            ),
            folder("packages", &[("ui", 22.0), ("charts", 17.0), ("auth", 6.0), ("billing", 8.0)]),
            folder("assets", &[("video", 240.0), ("images", 86.0), ("fonts", 11.0)]),
            folder(".git", &[("objects", 302.0), ("lfs", 140.0)]),
        ],
    );

    view! {
        <BubblePackChart
            root=repo
            label="Disk usage by folder"
            format=|v: f64| format!("{v:.0} MB")
        />
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bubble-pack-chart",
    name: "Bubble pack chart",
    group: "Parts of a whole",
    summary: "A tree as nested circles, each leaf's area its value, packed tightly inside its parent. Click a circle with children to zoom in; the breadcrumb or outer circle zooms out.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("bubble_pack_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bubble_pack_chart.rs"),
    parts: &[
        &["BubblePackChart", "root", "figure with a screen-reader table of every node"],
        &["", "breadcrumb", "Path from the root to the current zoom; earlier steps are buttons"],
        &["", "marks", "Focusable SVG; arrow keys move between circles"],
        &["", "bubble", "One circle; `data-depth`, `data-leaf`, `data-active`"],
        &["", "tooltip", "Name, value and share of the zoomed node"],
        &["", "legend", "Top-level branches when there are two or more"],
    ],
    keys: &[
        &["Arrow keys", "Move between circles in tree order"],
        &["Home / End", "Jump to the first or last circle"],
        &["Enter / Space", "Zoom into the focused circle, or out from the outer one"],
        &["Escape", "Clear the focused circle, then zoom out"],
    ],
    props: &[
        &["root", "Tree", "Required. Branch values come from their children"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Diameter in px; scales down to fit. Default 480"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip and table"],
    ],
    events: crate::doc::NODE_EVENTS,
};
