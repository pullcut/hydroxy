use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::confidence_band_chart::*;

pub fn demo() -> AnyView {
    // Conversion rate (%) for a new checkout, with a 95% interval that narrows as traffic builds.
    let rate = [2.4, 2.9, 3.1, 3.0, 3.3, 3.2, 3.4, 3.5, 3.4, 3.6];
    let spread = [0.9, 0.7, 0.6, 0.5, 0.45, 0.4, 0.35, 0.32, 0.3, 0.28];
    let weeks: Vec<Interval> = rate.iter().zip(spread).enumerate().map(|(i, (&v, s))| Interval::new(format!("Week {}", i + 1), v, v - s, v + s)).collect();

    view! {
        <ChartFrame title="New checkout conversion" description="Weekly estimate with its 95% interval. The dashed line is the old checkout's rate.">
            <ConfidenceBandChart
                data=weeks
                label="Weekly conversion rate of the new checkout, with 95% interval"
                value_label="Conversion"
                format=|v: f64| format!("{v:.1}%")
                annotations=vec![Annotation::y_line(2.8, "Old checkout")]
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "confidence-band-chart",
    name: "Confidence band chart",
    group: "Lines and areas",
    summary: "An estimate over time with a shaded band where the true value probably lies. Narrow bands mean more certainty.",
    demo,
    leptos: include_str!("confidence_band_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/confidence_band_chart.rs"),
    parts: &[
        &["ConfidenceBandChart", "root", "figure with a screen-reader table of estimates and bounds"],
        &["", "band", "Shaded region from the low to the high bound"],
        &["", "line", "The estimate"],
        &["", "crosshair", "Vertical line at the active point"],
        &["", "tooltip", "The estimate and its interval"],
        &["", "legend", "The estimate and the band"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next point"],
        &["Home / End", "First or last point"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Interval>", "Required. Interval::new(label, value, low, high) in x order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["value_label", "String", "Names the line. Default \"Estimate\""],
        &["band_label", "String", "Names the band. Default \"95% interval\""],
        &["curve", "Curve", "Linear (default) or Monotone"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats the ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "x is a point index"],
    ],
};
