use leptos::prelude::*;
use hydroxy_leptos::timeline_chart::{Item, Lane, TimelineChart, ms_from_civil};

pub fn demo() -> AnyView {
    let day = |m: i64, d: i64| ms_from_civil(2026, m, d);
    let lanes = vec![
        Lane::new(
            "Design",
            [Item::span("Research", day(10, 5), day(10, 23)), Item::span("Prototype", day(10, 19), day(11, 6)), Item::event("Design review", day(11, 9))],
        ),
        Lane::new("Build", [Item::span("API", day(10, 26), day(12, 4)), Item::span("Web app", day(11, 9), day(12, 18))]),
        Lane::new("Launch", [Item::span("Beta", day(12, 7), day(12, 21)), Item::event("Public launch", ms_from_civil(2027, 1, 12))]),
    ];

    view! { <TimelineChart lanes=lanes label="Release plan" /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "timeline-chart",
    name: "Timeline chart",
    group: "Time",
    summary: "Plan or review work over time. Items sit in lanes on a date axis with calendar-aligned ticks; spans are bars, events are diamonds, and overlapping items in a lane stack. Hover, touch or arrow-key to an item to read its dates.",
    demo,
    leptos: include_str!("timeline_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/timeline_chart.rs"),
    parts: &[
        &["TimelineChart", "root", "figure with a screen-reader table of every item"],
        &["", "body", "Focusable group of lanes; arrow keys step through items"],
        &["", "lane", "A named row group; grows to fit stacked items"],
        &["", "item", "Span or event; `data-active` and its tooltip shown when picked"],
        &["", "marker", "Diamond for an event"],
        &["", "axis", "Date ticks on whole hours, days, Mondays, months or years"],
        &["", "sr-live", "Announces the picked item's lane, name and dates"],
        &["", "sr-table", "Every item with its lane and dates"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step to the next or previous item, lane by lane"],
        &["Home / End", "Jump to the first or last item"],
        &["Escape", "Clear the picked item"],
    ],
    props: &[
        &["lanes", "Vec<Lane>", "Required. Lane::new(name, items)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["format", "Fn(f64) -> String", "Formats times in the tooltip, live region and table. Default: \"5 Oct 2026\""],
        &["Item::span", "(label, start, end)", "A range. Times are epoch milliseconds, UTC; see ms_from_civil"],
        &["Item::event", "(label, at)", "A single moment"],
    ],
};
