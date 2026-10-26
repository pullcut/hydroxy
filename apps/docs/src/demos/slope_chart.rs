use leptos::prelude::*;
use oxide_leptos::slope_chart::*;

pub fn demo() -> AnyView {
    // Each series has two values: before and after.
    let scores = vec![
        Series::new("North", [72.0, 81.0]),
        Series::new("South", [64.0, 61.0]),
        Series::new("East", [58.0, 74.0]),
        Series::new("West", [77.0, 76.0]),
        Series::new("Central", [66.0, 69.0]),
    ];

    view! {
        <SlopeChart series=scores from_label="2023" to_label="2024" label="Satisfaction score by region" format=|v: f64| format!("{v:.0}%") />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "slope-chart",
    name: "Slope chart",
    group: "Charts",
    summary: "Compare how several items changed between two points in time. All lines share one scale, and labels are nudged apart so close values stay readable.",
    demo,
    leptos: include_str!("slope_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/slope_chart.rs"),
    parts: &[
        &["SlopeChart", "root", "figure with a screen-reader table of both values"],
        &["", "series", "Focusable group per item; hover or focus quiets the others"],
        &["", "line", "Joins the two values; dashed when unchanged"],
        &["", "point", "Marker at each end"],
        &["", "name", "Direct labels with the value at each end"],
        &["", "column-label", "Heading over each axis"],
    ],
    keys: &[&["Tab", "Move between lines"]],
    props: &[
        &["series", "Vec<Series>", "Required. Series::new(name, [from, to])"],
        &["from_label", "String", "Required. Heading over the left axis"],
        &["to_label", "String", "Required. Heading over the right axis"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["format", "Fn(f64) -> String", "Formats values in labels and the table"],
    ],
};
