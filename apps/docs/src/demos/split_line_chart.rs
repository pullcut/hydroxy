use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::split_line_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    // Monthly revenue in £ thousands: actual to July, then the plan.
    let revenue = vec![
        Datum::new("Jan", 118.0),
        Datum::new("Feb", 124.0),
        Datum::new("Mar", 131.0),
        Datum::new("Apr", 129.0),
        Datum::new("May", 140.0),
        Datum::new("Jun", 148.0),
        Datum::new("Jul", 155.0),
        Datum::new("Aug", 161.0),
        Datum::new("Sep", 168.0),
        Datum::new("Oct", 174.0),
        Datum::new("Nov", 181.0),
        Datum::new("Dec", 190.0),
    ];

    view! {
        <ChartFrame title="Revenue" description="Actual to July, then the plan for the rest of the year.">
            <SplitLineChart
                data=revenue
                split=7_usize
                label="Monthly revenue, actual and projected"
                projected_label="Plan"
                format=|v: f64| format!("£{v:.0}k")
                annotations=vec![Annotation::y_line(150.0, "Break-even")]
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "split-line-chart",
    name: "Split line chart",
    group: "Lines and areas",
    summary: "One line that changes style at a point: solid for what happened, dashed for what is expected. The two halves meet at the last actual value.",
    demo,
    leptos: include_str!("split_line_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/split_line_chart.rs"),
    parts: &[
        &["SplitLineChart", "root", "figure with a screen-reader table marking each value actual or projected"],
        &["", "line", "Solid up to the split; data-dashed after it"],
        &["", "crosshair", "Vertical line at the active point"],
        &["", "dot", "The active point"],
        &["", "tooltip", "The value and whether it is actual or projected"],
        &["", "legend", "Actual and projected"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next point"],
        &["Home / End", "First or last point"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Datum::new(label, value), actual first"],
        &["split", "usize", "Required. Index of the first projected value"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["actual_label / projected_label", "String", "Default \"Actual\" and \"Projected\""],
        &["curve", "Curve", "Linear (default) or Monotone"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats the ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "x is a point index"],
    ],
    events: crate::doc::CHART_EVENTS,
};
