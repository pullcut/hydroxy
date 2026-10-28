use leptos::prelude::*;
use hydroxy_leptos::bar_chart::{BarChart, Datum};
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::segmented::{Segmented, SegmentedItem, SegmentedOption};

pub fn demo() -> AnyView {
    let weeks = vec![
        Datum::new("2 Sep", 6.0),
        Datum::new("9 Sep", 9.0),
        Datum::new("16 Sep", 7.0),
        Datum::new("23 Sep", 12.0),
        Datum::new("30 Sep", 10.0),
        Datum::new("7 Oct", 14.0),
    ];
    let (mode, set_mode) = signal(Some(String::from("ready")));
    let loading = Signal::derive(move || mode.get().as_deref() == Some("loading"));
    let empty = Signal::derive(move || mode.get().as_deref() == Some("empty"));

    view! {
        <div style="display: grid; gap: var(--space-5);">
            <Segmented
                label="State"
                items=vec![SegmentedOption::new("ready"), SegmentedOption::new("loading"), SegmentedOption::new("empty")]
                value=mode
                on_value_change=move |v| set_mode.set(Some(v))
            >
                <SegmentedItem value="ready" label="Ready" />
                <SegmentedItem value="loading" label="Loading" />
                <SegmentedItem value="empty" label="Empty" />
            </Segmented>
            // The frame owns the title, description and the loading and empty states.
            // export adds a button that downloads the chart's data table as CSV.
            <ChartFrame
                title="Interviews read"
                description="Per week, since the start of September."
                export=true
                loading=loading
                empty=empty
                empty_text="No interviews in this period"
            >
                <BarChart data=weeks label="Interviews read per week" />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "chart-frame",
    name: "Chart frame",
    group: "KPIs and frames",
    summary: "The shell around any chart: a title, a one-line description, an optional shared legend, a CSV download, and the loading and empty states. The chart stays mounted underneath, so switching states never replays its entrance.",
    demo,
    leptos: include_str!("chart_frame.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/chart_frame.rs"),
    parts: &[
        &["ChartFrame", "root", "section card; data-state is ready, loading or empty"],
        &["", "header", "Title and description"],
        &["", "actions", "Header buttons; export adds \"Download CSV\""],
        &["", "chart", "Wraps the chart; hidden while loading or empty"],
        &["", "loading", "Pulsing placeholder with role=status; the root gets aria-busy"],
        &["", "empty", "Dashed placeholder with the empty message"],
        &["", "legend", "Optional shared legend in the categorical colour order"],
    ],
    keys: &[],
    props: &[
        &["title", "String", "Heading"],
        &["description", "String", "One line saying what the numbers are"],
        &["legend", "Vec<String>", "Series names for a shared legend. Leave empty when the chart has its own"],
        &["loading", "bool", "Replace the chart with a placeholder. Wins over empty"],
        &["empty", "bool", "Replace the chart with empty_text"],
        &["empty_text", "String", "Default \"No data for this period\""],
        &["placeholder_height", "u16", "Height in px of the loading and empty placeholder. Default 180"],
        &["export", "bool", "Show a Download CSV button that saves the chart's data table, named after the title"],
    ],
};
