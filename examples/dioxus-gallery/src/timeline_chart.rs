use dioxus::prelude::*;
use hydroxy_dioxus::timeline_chart::{Item, Lane, TimelineChart, ms_from_civil};

#[component]
pub fn TimelineChartDemo() -> Element {
    let day = |m: i64, d: i64| ms_from_civil(2026, m, d);
    let lanes = vec![
        Lane::new(
            "Design",
            [
                Item::span("Research", day(10, 5), day(10, 23)),
                Item::span("Prototype", day(10, 19), day(11, 6)),
                Item::event("Design review", day(11, 9)),
            ],
        ),
        Lane::new(
            "Build",
            [Item::span("API", day(10, 26), day(12, 4)), Item::span("Web app", day(11, 9), day(12, 18))],
        ),
        Lane::new(
            "Launch",
            [Item::span("Beta", day(12, 7), day(12, 21)), Item::event("Public launch", ms_from_civil(2027, 1, 12))],
        ),
    ];

    rsx! {
        TimelineChart { lanes, label: "Release plan" }
    }
}
