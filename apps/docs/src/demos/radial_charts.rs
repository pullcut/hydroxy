use hydroxy_leptos::button::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::radial_chart::*;
use leptos::prelude::*;

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

    let arcs = goals.clone();
    // Low load is good here, so the bands run green to red.
    let load_bands = || {
        vec![
            Threshold::new(0.0, "var(--status-success)"),
            Threshold::new(60.0, "var(--status-warning)"),
            Threshold::new(85.0, "var(--status-danger)"),
        ]
    };
    // The needle swings to each new reading and settles.
    let tick = RwSignal::new(0_usize);
    let load = Signal::derive(move || [47.0, 72.0, 91.0, 28.0][tick.get() % 4]);

    // A changed value glides along the arc.
    let week = RwSignal::new(0_usize);
    let backlog = Signal::derive(move || [38.0, 64.0, 91.0][week.get() % 3]);

    view! {
        <div style="display: grid; gap: var(--space-8);">
            <ChartFrame title="Support SLA met" description="Share of tickets answered inside the target time.">
                <div style="display: flex; flex-wrap: wrap; gap: var(--space-8);">
                    <GaugeChart value=94.0 label="SLA met this week" caption="this week" thresholds=bands() format=|v: f64| format!("{v:.0}%") />
                    <GaugeChart value=78.0 label="SLA met last week" caption="last week" thresholds=bands() format=|v: f64| format!("{v:.0}%") />
                    <GaugeChart value=52.0 label="SLA met two weeks ago" caption="two weeks ago" thresholds=bands() format=|v: f64| format!("{v:.0}%") />
                </div>
            </ChartFrame>
            <ChartFrame title="Backlog cleared" description="Step through the last three sprints.">
                <div style="display: flex; align-items: center; flex-wrap: wrap; gap: var(--space-6);">
                    <GaugeChart value=backlog label="Backlog cleared this sprint" caption="of planned work" thresholds=bands() format=|v: f64| format!("{v:.0}%") />
                    <Button variant=ButtonVariant::Secondary on_click=move |_| week.update(|w| *w += 1)>"Next sprint"</Button>
                </div>
            </ChartFrame>
            <ChartFrame title="Server load" description="One reading in three gauge styles: arc, segmented status bands and needle.">
                <div style="display: flex; flex-wrap: wrap; gap: var(--space-8);">
                    <GaugeChart value=load label="CPU load, arc" caption="CPU" thresholds=load_bands() format=|v: f64| format!("{v:.0}%") />
                    <GaugeChart value=load label="CPU load, segmented" caption="CPU" thresholds=load_bands() variant=GaugeVariant::Segmented format=|v: f64| format!("{v:.0}%") />
                    <GaugeChart value=load label="CPU load, needle" caption="CPU" thresholds=load_bands() variant=GaugeVariant::Needle format=|v: f64| format!("{v:.0}%") />
                </div>
                <Button variant=ButtonVariant::Secondary on_click=move |_| tick.update(|t| *t += 1)>"New reading"</Button>
            </ChartFrame>
            <ChartFrame title="Research goals" description="Progress against a target of 100 each.">
                <RadialBarChart data=goals max=100.0 label="Progress towards each research goal" />
            </ChartFrame>
            <ChartFrame title="Research goals, as arcs" description="Three-quarter arcs with each label at its arc's start.">
                <RadialBarChart data=arcs max=100.0 sweep=270.0 size=240 label="Progress towards each research goal, as arcs" />
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
    group: "Radial",
    summary: "GaugeChart shows one value on an arc in three styles: a thermometer fill coloured by the threshold band it has reached, segmented status bands with a pointer, or a needle that sweeps to the value and settles. RadialBarChart stacks progress rings, the first outermost; below a full turn each ring is an arc labelled at its start. Hover, touch or focus either to read the value in a tooltip. For parts of a whole, use the donut chart.",
    demo,
    leptos: include_str!("radial_charts.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/radial_charts.rs"),
    parts: &[
        &[
            "GaugeChart",
            "root",
            "Focusable role=meter with the value, range and a text value; `data-active` while read",
        ],
        &["", "track", "Full three-quarter arc behind the value"],
        &["", "value-arc", "Filled share, in the colour of the active threshold band"],
        &["", "mark", "A tick where each threshold starts"],
        &["", "center", "The value, with an optional caption"],
        &[
            "",
            "segment",
            "Segmented and needle styles: one threshold band; segmented lights the band holding the value",
        ],
        &["", "pointer", "Segmented style: the marker at the value"],
        &["", "needle", "Needle style: the arm and hub, turned to the value"],
        &["", "tooltip", "Value and threshold band, at the tip of the arc"],
        &["RadialBarChart", "root", "figure; `data-active` while a ring is picked"],
        &["", "plot", "Focusable group holding the rings and tooltip"],
        &["", "ring", "Track and fill for one value; hover, touch or keys pick it out with its legend row"],
        &["", "ring-label", "The ring's name at its arc start, when sweep is 270 or less"],
        &["", "tooltip", "Value and share of the full ring, at the end of its fill"],
        &["", "legend", "Swatch, label, value and share of the full ring"],
        &["Both", "sr-live", "Announces the value being read"],
        &["", "sr-table", "Every value with its band or share, for screen readers and CSV export"],
    ],
    keys: &[
        &["Tab", "GaugeChart: focus shows the tooltip"],
        &["ArrowRight / ArrowLeft", "RadialBarChart: next or previous ring"],
        &["Home / End", "RadialBarChart: first or last ring"],
        &["Escape", "Clear the tooltip"],
    ],
    props: &[
        &["value", "f64", "GaugeChart. Required. The current value"],
        &["min / max", "f64", "GaugeChart. The scale. Default 0 to 100"],
        &[
            "thresholds",
            "Vec<Threshold>",
            "GaugeChart. Threshold::new(from, css colour); the highest band reached wins",
        ],
        &["caption", "String", "GaugeChart. Text under the centre value"],
        &["variant", "GaugeVariant", "GaugeChart. Arc (default), Segmented or Needle"],
        &["data", "Vec<Datum>", "RadialBarChart. Required. One ring per datum, outermost first"],
        &["max", "f64", "RadialBarChart. What a full ring means. Default: the largest value"],
        &[
            "sweep",
            "f64",
            "RadialBarChart. Degrees a full ring covers. Default 360; at 270 or less labels sit at the arc starts",
        ],
        &["label", "String", "Required on both. What the chart shows; the accessible name"],
        &["size / thickness", "u16", "Diameter and arc width in px"],
        &["format", "Fn(f64) -> String", "Formats values, scale ends, tooltips and the table"],
    ],
};
