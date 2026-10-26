use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::radial_chart::*;

pub fn demo() -> AnyView {
    let goals = vec![
        Datum::new("Interviews", 42.0),
        Datum::new("Surveys", 28.0),
        Datum::new("Tickets", 65.0),
        Datum::new("Reviews", 12.0),
    ];
    // The arc takes the colour of the highest band the value has reached.
    let bands = || {
        vec![
            Threshold::new(0.0, "var(--status-danger)"),
            Threshold::new(70.0, "var(--status-warning)"),
            Threshold::new(90.0, "var(--status-success)"),
        ]
    };

    view! {
        <div style="display: grid; gap: var(--space-8);">
            <ChartFrame title="Support SLA met" description="Share of tickets answered inside the target time.">
                <div style="display: flex; flex-wrap: wrap; gap: var(--space-8);">
                    <GaugeChart value=94.0 label="SLA met this week" caption="this week" thresholds=bands() format=|v: f64| format!("{v:.0}%") />
                    <GaugeChart value=78.0 label="SLA met last week" caption="last week" thresholds=bands() format=|v: f64| format!("{v:.0}%") />
                    <GaugeChart value=52.0 label="SLA met two weeks ago" caption="two weeks ago" thresholds=bands() format=|v: f64| format!("{v:.0}%") />
                </div>
            </ChartFrame>
            <ChartFrame title="Research goals" description="Progress against a target of 100 each.">
                <RadialBarChart data=goals max=100.0 label="Progress towards each research goal" />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "radial-charts",
    name: "Radial charts",
    group: "Charts",
    summary: "GaugeChart shows one value on an arc, coloured by the threshold band it has reached. RadialBarChart stacks progress rings, the first outermost. For parts of a whole, use the donut chart.",
    demo,
    leptos: include_str!("radial_charts.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/radial_charts.rs"),
    parts: &[
        &["GaugeChart", "root", "role=meter with the value, range and a text value"],
        &["", "track", "Full three-quarter arc behind the value"],
        &["", "value-arc", "Filled share, in the colour of the active threshold band"],
        &["", "mark", "A tick where each threshold starts"],
        &["", "center", "The value, with an optional caption"],
        &["RadialBarChart", "root", "figure with a screen-reader table of every ring"],
        &["", "ring", "Track and fill for one value; hovering picks it out with its legend row"],
        &["", "legend", "Swatch, label, value and share of the full ring"],
    ],
    keys: &[],
    props: &[
        &["value", "f64", "GaugeChart. Required. The current value"],
        &["min / max", "f64", "GaugeChart. The scale. Default 0 to 100"],
        &["thresholds", "Vec<Threshold>", "GaugeChart. Threshold::new(from, css colour); the highest band reached wins"],
        &["caption", "String", "GaugeChart. Text under the centre value"],
        &["data", "Vec<Datum>", "RadialBarChart. Required. One ring per datum, outermost first"],
        &["max", "f64", "RadialBarChart. What a full ring means. Default: the largest value"],
        &["label", "String", "Required on both. What the chart shows; the accessible name"],
        &["size / thickness", "u16", "Diameter and arc width in px"],
        &["format", "Fn(f64) -> String", "Formats values and scale ends"],
    ],
};
