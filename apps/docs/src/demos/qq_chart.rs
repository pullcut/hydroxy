use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::qq_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // Checkout latency in ms: roughly normal, so the points follow the line.
    let latency = vec![
        181.0, 236.0, 214.0, 198.0, 229.0, 251.0, 205.0, 222.0, 190.0, 244.0, 217.0, 208.0, 263.0, 226.0, 199.0, 232.0,
        211.0, 186.0, 248.0, 220.0, 203.0, 238.0, 215.0, 227.0, 194.0, 257.0, 209.0, 231.0, 223.0, 212.0,
    ];
    // Session length in minutes: skewed right, so the top end curves away above the line.
    let sessions = vec![
        1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0, 6.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 13.0, 15.0,
        17.0, 20.0, 24.0, 29.0, 35.0, 44.0, 58.0,
    ];

    view! {
        <div style="display: grid; gap: var(--space-10); grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));">
            <ChartFrame title="Checkout latency" description="Close to normal: the points hug the line.">
                <QQChart sample=latency label="Normal Q-Q plot of checkout latency" value_label="Latency" format=|v: f64| format!("{v:.0} ms") />
            </ChartFrame>
            <ChartFrame title="Session length" description="Skewed right: long sessions bend up and away from the line.">
                <QQChart sample=sessions label="Normal Q-Q plot of session length" value_label="Session" format=|v: f64| format!("{v:.0} min") />
            </ChartFrame>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "qq-chart",
    name: "Q-Q chart",
    group: "Distributions",
    summary: "Check whether a sample looks normal. Each sorted value is plotted against the normal quantile it would have; a straight line of points means yes.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("qq_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/qq_chart.rs"),
    parts: &[
        &["QQChart", "root", "figure with a screen-reader table of rank, normal quantile and value"],
        &["", "dot", "One per sample value, smallest first"],
        &["", "reference", "Dashed line the points would follow if the sample were normal (mean + sd × quantile)"],
        &["", "x-label", "Normal quantiles in standard deviations"],
        &["", "tooltip", "The value and its normal quantile"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Next smaller or larger value"],
        &["Home / End", "Smallest or largest value"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["sample", "Vec<f64>", "Required. Any order; NaN is dropped"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["value_label", "String", "What the sample measures. Default \"Value\""],
        &["height", "u16", "Plot height in px. Default 260"],
        &["format", "Fn(f64) -> String", "Formats the value ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "x in standard deviations, y in sample units"],
    ],
    events: crate::doc::CHART_EVENTS,
};
