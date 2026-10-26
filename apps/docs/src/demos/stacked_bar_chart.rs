use leptos::prelude::*;
use oxide_leptos::stacked_bar_chart::*;

pub fn demo() -> AnyView {
    let weeks: Vec<String> = ["30 Sep", "7 Oct", "14 Oct", "21 Oct"].map(String::from).to_vec();
    let tickets = vec![
        Series::new("Email", [42.0, 38.0, 51.0, 47.0]),
        Series::new("Chat", [28.0, 35.0, 31.0, 40.0]),
        Series::new("Phone", [12.0, 9.0, 14.0, 11.0]),
    ];
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].map(String::from).to_vec();
    // Negative values stack down from zero; the total is the net change.
    let headcount = vec![
        Series::new("Hires", [8.0, 5.0, 11.0, 6.0]),
        Series::new("Transfers in", [2.0, 3.0, 1.0, 2.0]),
        Series::new("Leavers", [-3.0, -6.0, -2.0, -4.0]),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <StackedBarChart series=tickets labels=weeks label="Tickets by channel per week" />
            <StackedBarChart series=headcount labels=quarters label="Headcount change by quarter" height=180 />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "stacked-bar-chart",
    name: "Stacked bar chart",
    group: "Data",
    summary: "Show a total and what it is made of, across categories or weeks. Negative values stack below zero, so gains and losses read side by side.",
    demo,
    leptos: include_str!("stacked_bar_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/stacked_bar_chart.rs"),
    parts: &[
        &["StackedBarChart", "root", "figure with a screen-reader table of every value and total"],
        &["", "gridline", "Recessive gridlines at nice round ticks; the zero line is stronger"],
        &["", "column", "One per label; hovering brightens its stack"],
        &["", "segment", "One per series, in categorical colour, with a 1px gap between"],
        &["", "total", "Net total above the positive stack"],
        &["", "legend", "Series names; hovering one quiets the others"],
    ],
    keys: &[],
    props: &[
        &["series", "Vec<Series>", "Required. Series::new(name, values), stacked bottom-up in order"],
        &["labels", "Vec<String>", "Required. One label per column"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 200"],
        &["show_totals", "bool", "Show each column's total. Default true; otherwise on hover"],
        &["format", "Fn(f64) -> String", "Formats ticks, totals and the table"],
    ],
};
