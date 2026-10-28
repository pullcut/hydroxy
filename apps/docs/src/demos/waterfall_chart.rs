use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::waterfall_chart::*;

pub fn demo() -> AnyView {
    // An opening balance, the changes, then the closing total.
    let bridge = vec![
        Step::start("FY24 revenue", 4_200.0),
        Step::change("New customers", 1_150.0),
        Step::change("Expansion", 640.0),
        Step::change("Price rise", 310.0),
        Step::change("Churn", -820.0),
        Step::change("Discounts", -270.0),
        Step::total("FY25 revenue"),
    ];
    let cash = vec![
        Step::change("Jan", 120.0),
        Step::change("Feb", -45.0),
        Step::change("Mar", 80.0),
        Step::total("Q1"),
        Step::change("Apr", -160.0),
        Step::change("May", 35.0),
        Step::change("Jun", 90.0),
        Step::total("H1"),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Revenue bridge" description="From last year's revenue to this year's, in thousands.">
                <WaterfallChart steps=bridge label="Revenue bridge from FY24 to FY25" format=|v: f64| format!("£{v:.0}k") />
            </ChartFrame>
            <ChartFrame title="Net cash flow" description="Monthly changes with quarter and half-year totals.">
                <WaterfallChart
                    steps=cash
                    label="Net cash flow by month"
                    height=200
                    annotations=vec![Annotation::y_line(100.0, "Reserve floor")]
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "waterfall-chart",
    name: "Waterfall chart",
    group: "Bars",
    summary: "Show how a starting value becomes an end value. Each change floats from the running total, rises and falls in their own colours, with optional total bars.",
    demo,
    leptos: include_str!("waterfall_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/waterfall_chart.rs"),
    parts: &[
        &["WaterfallChart", "root", "figure with a screen-reader table of every change and running total"],
        &["", "bar", "One per step; data-kind is increase, decrease or total"],
        &["", "connector", "Dashed line carrying the running total to the next bar"],
        &["", "overlay", "Focusable plot that drives the tooltip"],
        &["", "tooltip", "The step's change and the running total after it"],
        &["", "annotation", "Reference lines, bands and points"],
        &["", "legend", "Increase, decrease and (when used) total colours"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next step"],
        &["Home / End", "First or last step"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["steps", "Vec<Step>", "Required. Step::change(label, value), Step::total(label), or Step::start(label, value) to set the running total"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and points; x is a step index"],
    ],
};
