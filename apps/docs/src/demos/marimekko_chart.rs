use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::marimekko_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let labels = vec!["North America".to_string(), "Europe".into(), "Asia Pacific".into(), "Latin America".into()];
    let series = vec![
        Series::new("Enterprise", [320.0, 180.0, 90.0, 20.0]),
        Series::new("Mid-market", [210.0, 160.0, 120.0, 35.0]),
        Series::new("Small business", [90.0, 110.0, 140.0, 45.0]),
    ];

    view! {
        <ChartFrame title="Market" description="Column width is each region's share of revenue; height splits it by segment.">
            <MarimekkoChart series=series labels=labels label="Revenue by region and segment" format=|v: f64| format!("${v:.0}m") />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "marimekko-chart",
    name: "Marimekko chart",
    group: "Bars",
    summary: "A variable-width 100% stacked chart. Each column is as wide as its category's share of the grand total, and its segments split it by series, so area reads as share of everything.",
    demo,
    leptos: include_str!("marimekko_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/marimekko_chart.rs"),
    parts: &[
        &["MarimekkoChart", "root", "figure with a screen-reader table of every value"],
        &["", "plot", "Focusable chart; arrow keys move between segments"],
        &["", "gridline", "0 to 100% in quarters"],
        &["", "column", "One per category, as wide as its share"],
        &["", "segment", "One per series, stacked from the bottom"],
        &["", "x-label", "Category name and total under its column"],
        &["", "tooltip", "Value, share of the column and of the total"],
        &["", "legend", "Series names, shown for two or more series"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous category"],
        &["ArrowUp / ArrowDown", "Series above or below"],
        &["Home / End", "First or last category"],
        &["Escape", "Clear the active segment"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. Series::new(name, values), one value per category"],
        &["labels", "Vec<String>", "Required. Category names, left to right"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 280"],
        &["format", "Fn(f64) -> String", "Formats totals, tooltip and table"],
    ],
};
