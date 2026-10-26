use leptos::prelude::*;
use oxide_leptos::timeline_chart::{Item, Lane, TimelineChart, ms_from_civil};

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
    group: "Charts",
    summary: "Plan or review work over time. Items sit in lanes on a date axis with calendar-aligned ticks; spans are bars, events are diamonds, and overlapping items in a lane stack.",
    demo,
    leptos: include_str!("timeline_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/timeline_chart.rs"),
    parts: &[
        &["TimelineChart", "root", "figure with a screen-reader table of every item"],
        &["", "lane", "A named row group; grows to fit stacked items"],
        &["", "item", "Focusable span or event; shows its dates on hover or focus"],
        &["", "marker", "Diamond for an event"],
        &["", "axis", "Date ticks on whole hours, days, Mondays, months or years"],
    ],
    keys: &[&["Tab", "Move between items"]],
    props: &[
        &["lanes", "Vec<Lane>", "Required. Lane::new(name, items)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["Item::span", "(label, start, end)", "A range. Times are epoch milliseconds, UTC; see ms_from_civil"],
        &["Item::event", "(label, at)", "A single moment"],
    ],
};
