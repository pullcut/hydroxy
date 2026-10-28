use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::pareto_chart::*;

pub fn demo() -> AnyView {
    // Any order: the chart sorts largest first.
    let causes = vec![
        Datum::new("Login problems", 84.0),
        Datum::new("Billing questions", 212.0),
        Datum::new("Slow exports", 47.0),
        Datum::new("Missing invoices", 129.0),
        Datum::new("Sync conflicts", 66.0),
        Datum::new("Permissions", 31.0),
        Datum::new("Other", 18.0),
    ];

    view! {
        <ChartFrame title="Tickets by cause" description="Two causes account for most of the volume. The line reads on the right-hand axis.">
            <ParetoChart data=causes label="Support tickets by cause" value_label="Tickets" />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "pareto-chart",
    name: "Pareto chart",
    group: "Bars",
    summary: "Find the few causes behind most of a total. Bars sort largest first and a line climbs to 100% on its own labelled right-hand axis, the one chart here with two.",
    demo,
    leptos: include_str!("pareto_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/pareto_chart.rs"),
    parts: &[
        &["ParetoChart", "root", "figure with a screen-reader table of values and cumulative shares"],
        &["", "bar", "One per category, largest first"],
        &["", "line", "Cumulative share of the total, through the bar centres"],
        &["", "dot", "One per category on the cumulative line"],
        &["", "gridline", "Left ticks for the bars; right ticks (data-axis=right) for 0-100%"],
        &["", "tooltip", "The category's value and the cumulative share up to it"],
        &["", "legend", "The bar measure and the cumulative line"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next category"],
        &["Home / End", "Largest or smallest category"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Datum::new(label, value), any order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["value_label", "String", "What the bars count. Default \"Count\""],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats the left ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "On the left axis; x is a position in sorted order"],
    ],
};
