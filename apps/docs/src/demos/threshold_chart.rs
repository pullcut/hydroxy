use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::threshold_chart::*;

pub fn demo() -> AnyView {
    // p95 API latency over two weeks, in milliseconds.
    let latency = vec![
        Datum::new("Oct 1", 240.0),
        Datum::new("Oct 2", 262.0),
        Datum::new("Oct 3", 251.0),
        Datum::new("Oct 4", 298.0),
        Datum::new("Oct 5", 334.0),
        Datum::new("Oct 6", 356.0),
        Datum::new("Oct 7", 312.0),
        Datum::new("Oct 8", 276.0),
        Datum::new("Oct 9", 259.0),
        Datum::new("Oct 10", 288.0),
        Datum::new("Oct 11", 321.0),
        Datum::new("Oct 12", 305.0),
        Datum::new("Oct 13", 270.0),
        Datum::new("Oct 14", 248.0),
    ];

    view! {
        <ChartFrame title="API latency" description="Daily p95 against the 300 ms objective. Days over it show in the first colour.">
            <ThresholdChart
                data=latency
                threshold=300.0
                label="Daily p95 API latency against the 300 ms objective"
                threshold_label="Objective"
                above_label="Over objective"
                below_label="Within objective"
                curve=Curve::Monotone
                format=|v: f64| format!("{v:.0} ms")
                annotations=vec![Annotation::x_band(4.0, 6.0, "Incident")]
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "threshold-chart",
    name: "Threshold chart",
    group: "Lines and areas",
    summary: "A line judged against a limit: one colour above it, another below, with the gap shaded. Good for objectives, budgets and alert levels.",
    demo,
    leptos: include_str!("threshold_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/threshold_chart.rs"),
    parts: &[
        &["ThresholdChart", "root", "figure with a screen-reader table of values and the threshold"],
        &["", "clip", "Two copies of the line and fill, clipped above (data-side=above) and below the threshold"],
        &["", "line", "The values"],
        &["", "area", "Fill between the line and the threshold"],
        &["", "threshold", "Dashed line at the threshold, with its label"],
        &["", "tooltip", "The value, the threshold and the difference"],
        &["", "legend", "Above and below"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next point"],
        &["Home / End", "First or last point"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Datum::new(label, value) in x order"],
        &["threshold", "f64", "Required. The value the line is judged against"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["threshold_label", "String", "Default \"Threshold\""],
        &["above_label / below_label", "String", "Legend names. Default \"Above\" and \"Below\""],
        &["curve", "Curve", "Linear (default) or Monotone"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats the ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "x is a point index"],
    ],
};
