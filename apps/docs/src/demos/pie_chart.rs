use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::pie_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let regions = vec![
        Datum::new("North America", 412_000.0),
        Datum::new("Europe", 298_000.0),
        Datum::new("Asia Pacific", 187_000.0),
        Datum::new("Latin America", 64_000.0),
    ];
    let browsers = vec![
        Datum::new("Chrome", 6420.0),
        Datum::new("Safari", 2310.0),
        Datum::new("Edge", 940.0),
        Datum::new("Firefox", 610.0),
        Datum::new("Samsung Internet", 180.0),
        Datum::new("Opera", 95.0),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Revenue" description="By region, last quarter.">
                <PieChart data=regions label="Revenue by region" format=|v: f64| format!("${:.0}k", v / 1000.0) />
            </ChartFrame>
            <ChartFrame title="Sessions" description="Leader lines keep small slices readable.">
                <PieChart data=browsers label="Sessions by browser" labels=PieLabels::Leader size=180 />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "pie-chart",
    name: "Pie chart",
    group: "Parts of a whole",
    summary: "Parts of a whole as a full pie, with shares inside the slices or on leader lines. The active slice pulls out. More than six parts fold into a grey \"Other\".",
    demo,
    leptos: include_str!("pie_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/pie_chart.rs"),
    parts: &[
        &["PieChart", "root", "figure with a screen-reader table of every part"],
        &["", "plot", "Focusable pie; arrow keys step through slices"],
        &["", "slice", "One per part, sweeping in from its start angle"],
        &["", "slice-label", "Share inside slices of 6% or more"],
        &["", "leader", "Line and label outside the pie, with labels=Leader"],
        &["", "tooltip", "Part, value and share of the active slice"],
        &["", "legend", "Part names; hovering one picks out its slice"],
    ],
    keys: &[
        &["ArrowRight / ArrowDown", "Next slice, clockwise"],
        &["ArrowLeft / ArrowUp", "Previous slice"],
        &["Home / End", "First or last slice"],
        &["Escape", "Clear the active slice"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Parts clockwise from 12 o'clock"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Pie diameter in px. Default 200"],
        &["labels", "PieLabels", "Inside (default), Leader or None"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip and table"],
    ],
};
