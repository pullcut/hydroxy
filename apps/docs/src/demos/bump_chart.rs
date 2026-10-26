use leptos::prelude::*;
use oxide_leptos::bump_chart::*;

pub fn demo() -> AnyView {
    let weeks: Vec<String> = ["Wk 1", "Wk 2", "Wk 3", "Wk 4", "Wk 5", "Wk 6"].map(String::from).to_vec();
    // Points scored each week; the chart ranks the teams at every step.
    let teams = vec![
        Series::new("Falcons", [42.0, 55.0, 61.0, 48.0, 70.0, 66.0]),
        Series::new("Otters", [51.0, 47.0, 58.0, 63.0, 52.0, 71.0]),
        Series::new("Herons", [38.0, 60.0, 49.0, 66.0, 64.0, 58.0]),
        Series::new("Badgers", [60.0, 44.0, 52.0, 41.0, 45.0, 49.0]),
        Series::new("Kestrels", [35.0, 39.0, 66.0, 55.0, 68.0, 62.0]),
    ];

    view! {
        // Hover a line, or Tab to it, to pick that team out.
        <BumpChart series=teams labels=weeks label="Team ranking by weekly points" />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bump-chart",
    name: "Bump chart",
    group: "Charts",
    summary: "Show how items trade places over time. Values only decide the order at each step, so the chart answers who is ahead, not by how much.",
    demo,
    leptos: include_str!("bump_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bump_chart.rs"),
    parts: &[
        &["BumpChart", "root", "figure with a screen-reader table of every rank and value"],
        &["", "marks", "The SVG; it scales to the container width"],
        &["", "series", "Focusable group per item; hover or focus quiets the others"],
        &["", "line", "S-curve between ranks, in the series colour"],
        &["", "point", "Marker at each step, with a native tooltip"],
        &["", "name", "Direct label at both ends, so no legend is needed"],
    ],
    keys: &[&["Tab", "Move between lines"]],
    props: &[
        &["series", "Vec<Series>", "Required. Series::new(name, values); values decide the rank"],
        &["labels", "Vec<String>", "Required. One label per column"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["higher_is_better", "bool", "Rank the highest value first. Default true"],
        &["format", "Fn(f64) -> String", "Formats values in tooltips and the table"],
    ],
};
