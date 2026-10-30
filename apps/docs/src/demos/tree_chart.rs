use hydroxy_leptos::tree_chart::*;
use leptos::prelude::*;

fn team(lead: &str, people: &[&str]) -> Tree {
    Tree::branch(lead, people.iter().map(|p| Tree::leaf(*p, 1.0)).collect())
}

#[component]
pub fn Demo() -> impl IntoView {
    // Reporting lines; each person counts as one, so a branch's value is its headcount.
    let org = Tree::branch(
        "Priya (CEO)",
        vec![
            Tree::branch("Marcus (CTO)", vec![team("Lena", &["Tom", "Aisha", "Ravi"]), team("Diego", &["Mei", "Sam"])]),
            team("Hannah (CFO)", &["Oliver", "Grace"]),
            Tree::branch("Kofi (COO)", vec![team("Ines", &["Yuki", "Ben", "Zara"]), Tree::leaf("Noah", 1.0)]),
        ],
    );

    view! {
        <div style="display: grid; gap: var(--space-8);">
            <TreeChart root=org.clone() label="Reporting lines" format=|v: f64| format!("{v:.0} people") />
            <TreeChart root=org label="Reporting lines, left to right" orientation=Orientation::Horizontal height=420 />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tree-chart",
    name: "Tree chart",
    group: "Relationships",
    summary: "A tidy node-link tree, top-down or left to right, with parents centred over their children. Click a branch to collapse or expand it, or move through it with the keys of a tree view.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("tree_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tree_chart.rs"),
    parts: &[
        &["TreeChart", "root", "figure with `data-orientation` and a screen-reader table of every node"],
        &["", "marks", "Focusable SVG; arrow keys move through the tree"],
        &["", "link", "Curved line from a parent to a child"],
        &["", "node", "Dot and label; `data-collapsed`, `data-leaf`, `data-active`"],
        &["", "tooltip", "Name, value and how many children are shown or hidden"],
        &["", "legend", "Top-level branches when there are two or more"],
    ],
    keys: &[
        &["Up / Down", "Previous or next visible node"],
        &["Right", "Expand a collapsed branch, else move to its first child"],
        &["Left", "Collapse an open branch, else move to the parent"],
        &["Home / End", "First or last visible node"],
        &["Enter / Space", "Collapse or expand the focused branch"],
        &["Escape", "Clear the focused node"],
    ],
    props: &[
        &["root", "Tree", "Required. Branch values come from their children"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["orientation", "Orientation", "Vertical (default, root at the top) or Horizontal (root on the left)"],
        &["height", "u16", "Plot height in px at an 800px drawing width. Default 360"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip and table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
