use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::grouped_bar_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].map(String::from).to_vec();
    let signups = vec![
        Series::new("Organic", [1_240.0, 1_410.0, 1_380.0, 1_620.0]),
        Series::new("Paid", [860.0, 1_020.0, 1_190.0, 1_080.0]),
        Series::new("Referral", [310.0, 360.0, 455.0, 520.0]),
    ];
    let regions: Vec<String> =
        ["North America", "Western Europe", "Asia Pacific", "Latin America"].map(String::from).to_vec();
    let margin = vec![Series::new("2024", [18.0, 12.5, 6.0, -4.5]), Series::new("2025", [21.0, 14.0, 9.5, 2.0])];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            // Hover a bar to read its quarter; focus the plot and use the arrow keys.
            <ChartFrame title="Signups by channel" description="Each quarter, side by side.">
                <GroupedBarChart
                    categories=quarters
                    series=signups
                    label="Signups by channel per quarter"
                    annotations=vec![Annotation::y_line(1_500.0, "Organic target")]
                />
            </ChartFrame>
            // Horizontal keeps long names readable; negatives hang left of zero.
            <ChartFrame title="Operating margin" description="By region, this year against last.">
                <GroupedBarChart
                    categories=regions
                    series=margin
                    label="Operating margin by region"
                    orientation=Orientation::Horizontal
                    height=260
                    format=|v: f64| format!("{v:.0}%")
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "grouped-bar-chart",
    name: "Grouped bar chart",
    group: "Bars",
    summary: "Compare a few series within each category, side by side from zero. Vertical or horizontal, with negative values hanging below the baseline.",
    demo,
    leptos: include_str!("grouped_bar_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/grouped_bar_chart.rs"),
    parts: &[
        &["GroupedBarChart", "root", "figure with a screen-reader table of every value"],
        &["", "gridline", "Recessive gridlines at nice round ticks; the zero line is stronger"],
        &["", "bar", "One per series and category, in categorical colour"],
        &["", "overlay", "Focusable plot that drives the tooltip"],
        &["", "tooltip", "The category and every series' value, the active one in bold"],
        &["", "annotation", "Reference lines, bands and points"],
        &["", "legend", "Series names and colours, shown for two or more series"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Next or previous category (vertical); next or previous series (horizontal)"],
        &["ArrowUp / ArrowDown", "Next or previous series (vertical); next or previous category (horizontal)"],
        &["Home / End", "First or last bar in the series"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["categories", "Vec<String>", "Required. One label per group"],
        &["series", "Vec<Series>", "Required. Series::new(name, values); past six, the smallest fold into Other"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["orientation", "Orientation", "Vertical (default) or Horizontal"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "Annotation::y_line, y_band, x_line, x_band, point; x is a category index"],
    ],
};
