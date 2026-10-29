use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::dot_plot_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let roles: Vec<String> =
        ["Product designer", "Frontend engineer", "Data analyst", "Support lead", "Engineering manager"]
            .map(String::from)
            .to_vec();
    // Two series draw a dumbbell: the bar is the change between years.
    let salaries =
        vec![Series::new("2024", [58.0, 64.0, 49.0, 41.0, 88.0]), Series::new("2025", [61.0, 70.0, 53.0, 42.5, 91.0])];
    let cities: Vec<String> = ["Lisbon", "Berlin", "Toronto", "Melbourne"].map(String::from).to_vec();
    let rent = vec![Series::new("Median rent", [1_350.0, 1_480.0, 2_150.0, 1_920.0])];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Median salary by role" description="Thousands per year, last year against this year.">
                <DotPlotChart rows=roles series=salaries label="Median salary by role, 2024 and 2025" format=|v: f64| format!("£{v:.0}k") />
            </ChartFrame>
            <ChartFrame title="One-bedroom rent" description="A single series is a plain dot plot.">
                <DotPlotChart
                    rows=cities
                    series=rent
                    label="Median one-bedroom rent by city"
                    annotations=vec![Annotation::y_line(1_800.0, "Budget")]
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "dot-plot-chart",
    name: "Dot plot chart",
    group: "Bars",
    summary: "Compare values across categories with dots instead of bars. Two or more series per row draw a dumbbell, so the gap between them reads at a glance.",
    demo,
    leptos: include_str!("dot_plot_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/dot_plot_chart.rs"),
    parts: &[
        &["DotPlotChart", "root", "figure with a screen-reader table of every value"],
        &["", "dot", "One per series and row, in categorical colour"],
        &["", "stick", "The dumbbell bar from a row's lowest to its highest dot"],
        &["", "x-label", "Row names, down the left"],
        &["", "tooltip", "The row and every series' value, the active one in bold"],
        &["", "legend", "Series names and colours, shown for two or more series"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Previous or next row"],
        &["ArrowLeft / ArrowRight", "Previous or next series in the row"],
        &["Home / End", "First or last series in the row"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["rows", "Vec<String>", "Required. One label per row"],
        &["series", "Vec<Series>", "Required. Series::new(name, values); past six, the smallest fold into Other"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog; the axis fits the data, not zero"],
        &["height", "u16", "Plot height in px. Default 36 per row"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "y_line and y_band mark values (drawn vertically); x is a row index"],
    ],
    events: crate::doc::CHART_EVENTS,
};
