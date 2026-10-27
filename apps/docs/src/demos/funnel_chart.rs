use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::funnel_chart::*;

pub fn demo() -> AnyView {
    let signup = vec![
        Datum::new("Visited pricing", 18_400.0),
        Datum::new("Started trial", 4_120.0),
        Datum::new("Invited a teammate", 1_870.0),
        Datum::new("Connected data", 1_240.0),
        Datum::new("Paid", 610.0),
    ];
    let hiring = vec![
        Datum::new("Applied", 640.0),
        Datum::new("Screened", 210.0),
        Datum::new("Interviewed", 72.0),
        Datum::new("Offered", 14.0),
        Datum::new("Hired", 11.0),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Signup funnel" description="Last 30 days, with conversion between stages.">
                <FunnelChart data=signup label="Signup funnel" />
            </ChartFrame>
            <ChartFrame title="Hiring" description="Engineering roles this year, left to right.">
                <FunnelChart data=hiring label="Hiring funnel" orientation=Orientation::Horizontal height=200 />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "funnel-chart",
    name: "Funnel chart",
    group: "Charts",
    summary: "Stages that narrow to the next, with the conversion from each stage to the one after. Runs top to bottom or left to right.",
    demo,
    leptos: include_str!("funnel_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/funnel_chart.rs"),
    parts: &[
        &["FunnelChart", "root", "figure with a screen-reader table of every stage"],
        &["", "plot", "Focusable funnel; arrow keys step through stages"],
        &["", "stage-label", "Stage name and value in the gutter"],
        &["", "stage", "Trapezoid from the stage's value down to the next one's"],
        &["", "conversion", "Share of the previous stage, on the seam between them"],
        &["", "tooltip", "Value, share of the previous stage and of the first"],
    ],
    keys: &[
        &["ArrowDown / ArrowRight", "Next stage"],
        &["ArrowUp / ArrowLeft", "Previous stage"],
        &["Home / End", "First or last stage"],
        &["Escape", "Clear the active stage"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Stages in order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["orientation", "Orientation", "Vertical (default) or Horizontal"],
        &["height", "u16", "Plot height in px. Default 280"],
        &["format", "Fn(f64) -> String", "Formats values in labels, tooltip and table"],
    ],
};
