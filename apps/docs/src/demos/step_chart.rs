use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::step_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let months: Vec<String> = (2022..=2024)
        .flat_map(|y| {
            ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
                .map(|m| format!("{m} {}", y % 100))
        })
        .collect();
    // The rate set at each month's meeting; it holds until the next change.
    let rate = vec![
        0.25, 0.5, 0.75, 0.75, 1.0, 1.25, 1.25, 1.75, 2.25, 2.25, 3.0, 3.5, 3.5, 4.0, 4.25, 4.25, 4.5, 5.0, 5.0, 5.25,
        5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.0, 5.0, 5.0, 4.75, 4.75,
    ];
    let events =
        vec![StepEvent::new(1, "First rise"), StepEvent::new(19, "Peak at 5.25%"), StepEvent::new(31, "First cut")];
    let quarters: Vec<String> =
        ["Q1 25", "Q2 25", "Q3 25", "Q4 25", "Q1 26", "Q2 26", "Q3 26", "Q4 26"].map(String::from).to_vec();
    let seats = vec![
        Series::new("Engineering", [42.0, 42.0, 48.0, 55.0, 55.0, 61.0, 61.0, 64.0]),
        Series::new("Support", [18.0, 22.0, 22.0, 22.0, 26.0, 26.0, 30.0, 30.0]),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Base rate" description="The policy rate after each monthly decision, January 2022 to December 2024.">
                <StepChart labels=months series=vec![Series::new("Base rate", rate)] events label="Base rate by month" format=|v: f64| format!("{v:.2}%") />
            </ChartFrame>
            // Headcount changes when a quarter closes, so the step lands before each point.
            <ChartFrame title="Headcount by team" description="People employed at the end of each quarter.">
                <StepChart labels=quarters series=seats mode=StepMode::Before label="Headcount by team and quarter" height=200 />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "step-chart",
    name: "Step chart",
    group: "Lines and areas",
    summary: "For values that hold and then jump, such as prices, rates and headcount. Lines step instead of sloping, so nobody reads a change as gradual, and labelled events mark why.",
    demo,
    leptos: include_str!("step_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/step_chart.rs"),
    parts: &[
        &["StepChart", "root", "figure with a screen-reader table; data-variant is after, before or middle"],
        &["", "line", "One series' steps, revealed left to right"],
        &["", "event", "A labelled marker that fades in after the lines"],
        &["", "event-label", "The event's text, above the plot"],
        &["", "crosshair", "Follows the point in force under the pointer"],
        &["", "tooltip", "Each series' value and any event at that point"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next point"],
        &["Home / End", "First or last point"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["labels", "Vec<String>", "Required. One x label per point"],
        &["series", "Vec<Series>", "Required. Series::new(name, values); more than six fold into Other"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["mode", "StepMode", "After (default), Before or Middle: where each jump falls"],
        &["events", "Vec<StepEvent>", "StepEvent::new(point index, label)"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and points; x is a point index"],
    ],
    events: crate::doc::CHART_EVENTS,
};
