use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::error_bar_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let variants = vec![
        Interval::new("Control", 4.1, 3.6, 4.6),
        Interval::new("Shorter form", 4.9, 4.3, 5.5),
        Interval::new("Social proof", 4.4, 3.8, 5.0),
        Interval::new("Free trial", 5.6, 4.8, 6.4),
    ];
    let teams = vec![
        Interval::new("Platform", 18.0, 14.5, 22.0),
        Interval::new("Payments", 26.0, 21.0, 31.5),
        Interval::new("Mobile", 21.5, 16.0, 28.0),
        Interval::new("Growth", 12.0, 9.5, 15.0),
        Interval::new("Data", 15.5, 12.0, 19.5),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Signup conversion" description="Each variant with its 95% confidence interval.">
                <ErrorBarChart
                    data=variants
                    label="Signup conversion by variant"
                    format=|v: f64| format!("{v:.1}%")
                    annotations=vec![Annotation::y_line(4.1, "Control")]
                />
            </ChartFrame>
            // Bars from zero, for counts or totals.
            <ChartFrame title="Cycle time" description="Median days from start to merge, with the interquartile range.">
                <ErrorBarChart data=teams label="Cycle time by team in days" bars=true height=220 />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "error-bar-chart",
    name: "Error bar chart",
    group: "Bars",
    summary: "Show estimates with their uncertainty. Each category gets a dot, or a bar from zero, and a capped whisker from its low to its high bound.",
    demo,
    leptos: include_str!("error_bar_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/error_bar_chart.rs"),
    parts: &[
        &[
            "ErrorBarChart",
            "root",
            "figure with a screen-reader table of estimates and bounds; data-variant is points or bars",
        ],
        &["", "dot", "The estimate (points)"],
        &["", "bar", "The estimate as a bar from zero (bars=true)"],
        &["", "whisker", "Capped line from the low to the high bound"],
        &["", "tooltip", "Estimate, low and high"],
        &["", "annotation", "Reference lines, bands and points"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next category"],
        &["Home / End", "First or last category"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Interval>", "Required. Interval::new(label, value, low, high)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["bars", "bool", "Draw bars from zero instead of dots. Default false"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and points; x is a category index"],
    ],
    events: crate::doc::CHART_EVENTS,
};
