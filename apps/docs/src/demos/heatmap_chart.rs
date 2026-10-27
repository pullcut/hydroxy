use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::heatmap_chart::HeatmapChart;

pub fn demo() -> AnyView {
    let days: Vec<String> = ["Mon", "Tue", "Wed", "Thu", "Fri"].map(String::from).to_vec();
    let hours: Vec<String> = ["9", "10", "11", "12", "13", "14", "15", "16"].map(String::from).to_vec();
    // Sessions per weekday and hour. A NaN is a cell with no data.
    let sessions = vec![
        vec![12.0, 24.0, 31.0, 18.0, 9.0, 22.0, 27.0, 14.0],
        vec![15.0, 29.0, 36.0, 21.0, 11.0, 26.0, 33.0, 17.0],
        vec![14.0, 27.0, 40.0, 25.0, 13.0, 30.0, 35.0, 19.0],
        vec![11.0, 23.0, 33.0, 19.0, f64::NAN, 24.0, 29.0, 13.0],
        vec![8.0, 16.0, 21.0, 12.0, 6.0, 10.0, 12.0, 5.0],
    ];

    view! {
        <ChartFrame title="Sessions by weekday and hour" description="Darker cells are busier. Hover a cell, or focus the grid and use the arrow keys.">
            <HeatmapChart
                values=sessions
                rows=days
                columns=hours
                label="Sessions by weekday and hour of day"
                show_values=true
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "heatmap-chart",
    name: "Heatmap chart",
    group: "Charts",
    summary: "A grid of values shaded from low to high. Use it for two categories against one measure, such as weekday against hour. The scale is one chart colour mixed into the surface, so it follows light and dark themes. Hover, touch or arrow-key to a cell to read it.",
    demo,
    leptos: include_str!("heatmap_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/heatmap_chart.rs"),
    parts: &[
        &["HeatmapChart", "root", "figure with a screen-reader table of every cell"],
        &["", "cells", "Focusable grid; arrow keys move between cells"],
        &["", "cell", "Shaded by --level; dashed when there is no data"],
        &["", "tooltip", "Row, column and value of the active cell"],
        &["", "scale", "Lowest and highest value with a colour ramp between"],
        &["", "sr-live", "Announces the active cell"],
        &["", "sr-table", "Every cell, one row per row label, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move along the row; the first press lands on the top-left cell"],
        &["ArrowUp / ArrowDown", "Move between rows"],
        &["Home / End", "First or last cell in the row"],
        &["Escape", "Clear the active cell"],
    ],
    props: &[
        &["values", "Vec<Vec<f64>>", "Required. One Vec per row. NaN or a short row leaves an empty cell"],
        &["rows", "Vec<String>", "Required. One label per row"],
        &["columns", "Vec<String>", "Required. One label per column"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["cell_height", "u16", "Cell height in px. Default 36"],
        &["show_values", "bool", "Print each value in its cell"],
        &["format", "Fn(f64) -> String", "Formats cells, tooltip, scale and table"],
    ],
};
