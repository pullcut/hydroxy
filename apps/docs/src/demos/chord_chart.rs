use leptos::prelude::*;
use hydroxy_leptos::chord_chart::*;

pub fn demo() -> AnyView {
    let plans: Vec<String> = ["Free", "Starter", "Team", "Business", "Enterprise"].iter().map(|s| (*s).to_string()).collect();
    // Accounts moving between plans over the year: row = from, column = to.
    let moves = vec![
        vec![0.0, 420.0, 160.0, 30.0, 4.0],
        vec![180.0, 0.0, 310.0, 60.0, 8.0],
        vec![40.0, 95.0, 0.0, 140.0, 22.0],
        vec![6.0, 12.0, 48.0, 0.0, 35.0],
        vec![0.0, 2.0, 5.0, 14.0, 0.0],
    ];

    view! { <ChordChart names=plans matrix=moves label="Accounts moving between plans" /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "chord-chart",
    name: "Chord chart",
    group: "Relationships",
    summary: "Flows between groups from a square matrix. Each arc is sized by what its group sends, and each ribbon is as wide at each end as the flow that way. Hover a group to see only its chords.",
    demo,
    leptos: include_str!("chord_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/chord_chart.rs"),
    parts: &[
        &["ChordChart", "root", "figure; `data-active` while a group or ribbon is picked out"],
        &["", "group", "Arc for one group, sized by its row total; `data-dim` when another group is active"],
        &["", "ribbon", "Flow between two groups, coloured by the side that sends more; `data-dim` when it isn't part of the active group"],
        &["", "tooltip", "Sends and receives for a group, or the flow each way for a ribbon"],
        &["", "sr-table", "The full matrix, for screen readers and CSV export"],
    ],
    keys: &[
        &["Arrow keys", "Step through groups"],
        &["Home / End", "First or last group"],
        &["Escape", "Clear the active group"],
    ],
    props: &[
        &["names", "Vec<String>", "Required. One name per group, in matrix order"],
        &["matrix", "Vec<Vec<f64>>", "Required. matrix[i][j] is the flow from group i to group j"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["pad", "f64", "Gap between groups in radians. Default 0.04"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip, readout and table"],
    ],
};
