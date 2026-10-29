use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::donut_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let channels = vec![
        Datum::new("Email", 182.0),
        Datum::new("Chat", 124.0),
        Datum::new("Phone", 61.0),
        Datum::new("Forms", 43.0),
    ];
    // Eight sources: the smallest fold into "Other", which stays grey.
    let sources = vec![
        Datum::new("Interviews", 12.0),
        Datum::new("Support tickets", 9.0),
        Datum::new("Surveys", 7.0),
        Datum::new("Sales notes", 5.0),
        Datum::new("NPS", 4.0),
        Datum::new("App reviews", 2.0),
        Datum::new("Forums", 1.0),
        Datum::new("Social", 1.0),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Tickets" description="By channel, this quarter.">
                <DonutChart data=channels label="Tickets by channel" center_label="Tickets" />
            </ChartFrame>
            <ChartFrame title="Sources" description="More than six fold into a grey Other.">
                <DonutChart data=sources label="Sources by type" center_label="Sources" size=150 thickness=18 />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "donut-chart",
    name: "Donut chart",
    group: "Parts of a whole",
    summary: "Parts of a whole. Up to six parts in the chart colour order; more fold into a grey \"Other\". Hover, touch or focus a segment or legend row to pick it out and read its value and share.",
    demo,
    leptos: include_str!("donut_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/donut_chart.rs"),
    parts: &[
        &["DonutChart", "root", "figure labelled by `label`; `data-active` while a segment is picked"],
        &["", "ring", "Focusable group holding the segments, centre and tooltip"],
        &["", "segment", "Ring slice with a 2px gap to its neighbours; `data-active` when picked"],
        &["", "center", "Total, or `center_value`, with an optional caption"],
        &["", "tooltip", "Name, value and share of the picked segment, at its arc"],
        &["", "legend", "Swatch, label, value and share for every part"],
        &["", "sr-live", "Announces the picked segment"],
        &["", "sr-table", "Every part with its value and share, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step to the next or previous segment"],
        &["Home / End", "Jump to the first or last segment"],
        &["Escape", "Clear the picked segment"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Parts in display order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Ring diameter in px. Default 180"],
        &["thickness", "u16", "Ring width in px. Default 22"],
        &["center_label", "String", "Caption under the centre figure"],
        &["center_value", "String", "Centre figure. Default: the total"],
        &["format", "Fn(f64) -> String", "Formats the centre and legend values"],
    ],
    events: crate::doc::CHART_EVENTS,
};
