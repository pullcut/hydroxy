use leptos::prelude::*;
use hydroxy_leptos::radar_chart::*;

pub fn demo() -> AnyView {
    let axes: Vec<String> = ["Speed", "Reach", "Reliability", "Support", "Docs", "Value"].map(String::from).to_vec();
    let vendors = vec![
        Series::new("Acme", [82.0, 64.0, 90.0, 55.0, 70.0, 48.0]),
        Series::new("Globex", [58.0, 88.0, 62.0, 80.0, 52.0, 76.0]),
    ];

    view! {
        <RadarChart series=vendors axes=axes label="Vendor comparison" max=100.0 />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "radar-chart",
    name: "Radar chart",
    group: "Radial",
    summary: "Compare a few items across the same handful of measures. Every axis starts at zero and shares one scale, so use it only when the measures are comparable. Hover, touch or focus a vertex to read it; hover a legend item to pick its series out.",
    demo,
    leptos: include_str!("radar_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/radar_chart.rs"),
    parts: &[
        &["RadarChart", "root", "figure holding the plot, legend, live region and table"],
        &["", "plot", "Focusable group around the SVG and tooltip"],
        &["", "ring", "Guide polygons at even steps, with value ticks along the first axis"],
        &["", "spoke", "One per axis, with its label at the end"],
        &["", "series", "Group per item; `data-dim` while another series is picked"],
        &["", "area", "The item's polygon, lightly filled"],
        &["", "point", "Vertex per axis; `data-active` when picked"],
        &["", "hit", "Transparent target around each vertex for hover and touch"],
        &["", "tooltip", "Series, axis and value of the picked vertex"],
        &["", "legend", "Names and colours, shown for two or more series"],
        &["", "sr-live", "Announces the picked vertex"],
        &["", "sr-table", "Every value by axis and series, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move to the previous or next axis"],
        &["ArrowUp / ArrowDown", "Move to the previous or next series"],
        &["Home / End", "Jump to the first or last axis"],
        &["Escape", "Clear the picked vertex"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. One value per axis; up to four stay legible"],
        &["axes", "Vec<String>", "Required. Axis names, clockwise from the top"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["max", "f64", "Value at the outer ring. Default: the data maximum, rounded up"],
        &["rings", "usize", "Guide rings. Default 4"],
        &["format", "Fn(f64) -> String", "Formats ticks, the tooltip and the table"],
    ],
};
