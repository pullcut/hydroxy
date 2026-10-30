use hydroxy_leptos::slope_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
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
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "slope-chart",
    name: "Slope chart",
    group: "Lines and areas",
    summary: "Compare how several items changed between two points in time. All lines share one scale, and labels are nudged apart so close values stay readable. Hover, touch or focus a line to read both values and the change; hover a legend item to pick its series out.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("slope_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/slope_chart.rs"),
    parts: &[
        &["SlopeChart", "root", "figure holding the plot, legend, live region and table"],
        &["", "plot", "Focusable group around the SVG and tooltip"],
        &["", "series", "Group per item; `data-dim` while another series is picked"],
        &["", "line", "Joins the two values; dashed when unchanged"],
        &["", "point", "Marker at each end; `data-active` when picked"],
        &["", "hit", "Transparent target around each end for hover and touch"],
        &["", "name", "Direct labels with the value at each end"],
        &["", "column-label", "Heading over each axis"],
        &["", "tooltip", "Both values and the change, at the picked end"],
        &["", "legend", "Names and colours, shown for two or more series"],
        &["", "sr-live", "Announces the picked series"],
        &["", "sr-table", "Every series with both values, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move between the two ends"],
        &["ArrowUp / ArrowDown", "Move to the previous or next series"],
        &["Home / End", "Jump to the first or last end"],
        &["Escape", "Clear the picked series"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. Series::new(name, [from, to])"],
        &["from_label", "String", "Required. Heading over the left axis"],
        &["to_label", "String", "Required. Heading over the right axis"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["format", "Fn(f64) -> String", "Formats values in labels, the tooltip and the table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
