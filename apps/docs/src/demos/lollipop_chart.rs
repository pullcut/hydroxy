use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::lollipop_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let nps = vec![
        Datum::new("Onboarding", 42.0),
        Datum::new("Billing", 18.0),
        Datum::new("Search", 31.0),
        Datum::new("Reporting", 55.0),
        Datum::new("Mobile app", 12.0),
        Datum::new("Integrations", 37.0),
    ];
    // Values spanning four orders of magnitude read better on a log axis.
    let downloads = vec![
        Datum::new("Core library", 412_000.0),
        Datum::new("Command line tool", 38_500.0),
        Datum::new("Editor plugin", 9_200.0),
        Datum::new("Migration helper", 1_150.0),
        Datum::new("Legacy adapter", 140.0),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Satisfaction by area" description="Net promoter score from last month's survey.">
                <LollipopChart data=nps label="Net promoter score by product area" annotations=vec![Annotation::y_line(30.0, "Goal")] />
            </ChartFrame>
            <ChartFrame title="Monthly downloads" description="Log scale, so the small packages stay visible.">
                <LollipopChart
                    data=downloads
                    label="Monthly downloads by package"
                    orientation=Orientation::Horizontal
                    scale=ScaleKind::Log
                    format=|v: f64| { if v >= 1000.0 { format!("{:.0}k", v / 1000.0) } else { format!("{v:.0}") } }
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "lollipop-chart",
    name: "Lollipop chart",
    group: "Bars",
    summary: "A lighter bar chart: a thin stick from zero and a dot at each value. Less ink than bars when there are many categories, and it works on a log axis.",
    demo,
    leptos: include_str!("lollipop_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/lollipop_chart.rs"),
    parts: &[
        &["LollipopChart", "root", "figure with a screen-reader table of every value"],
        &["", "stick", "Thin line from the baseline to the value"],
        &["", "dot", "The value, in the series colour; ringed when active"],
        &["", "overlay", "Focusable plot that drives the tooltip"],
        &["", "tooltip", "The category and its value"],
        &["", "annotation", "Reference lines, bands and points"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next value (Up/Down when horizontal)"],
        &["Home / End", "First or last value"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Datum::new(label, value); NaN leaves a gap"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["orientation", "Orientation", "Vertical (default) or Horizontal"],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["height", "u16", "Plot height in px. Default 220"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and points; x is a category index"],
    ],
    events: crate::doc::CHART_EVENTS,
};
