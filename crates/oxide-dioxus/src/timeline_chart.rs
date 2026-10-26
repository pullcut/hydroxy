//! `TimelineChart`: spans and events in lanes on a date axis. Placement and calendar
//! ticks come from `oxide_charts::timeline`; layout is plain CSS percentages.

use dioxus::prelude::*;
use oxide_charts::timeline;

pub use oxide_charts::timeline::{DAY, HOUR, Item, Lane, ms_from_civil};

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

fn when(i: &Item) -> String {
    if i.is_event() { timeline::format_date(i.start) } else { format!("{} to {}", timeline::format_date(i.start), timeline::format_date(i.end)) }
}

/// Time ranges and moments, one row group per lane. Overlapping items in a lane stack.
/// Times are milliseconds since the Unix epoch (UTC); see [`ms_from_civil`].
/// Items are focusable and show their dates on hover or focus.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TimelineChart(
    /// The lanes, top to bottom.
    lanes: ReadSignal<Vec<Lane>>,
    /// What the chart shows, such as "Release plan". Used as the accessible name.
    label: String,
) -> Element {
    let lanes_v = lanes();
    let layout = timeline::layout_lanes(&lanes_v, 6);
    let mut offset = 0;
    let mut placed_by_lane = Vec::new();
    for lane in &lanes_v {
        placed_by_lane.push(&layout.items[offset..offset + lane.items.len()]);
        offset += lane.items.len();
    }

    rsx! {
        figure { "data-scope": "timeline-chart", "data-part": "root",
            div { "data-part": "body", role: "group", "aria-label": "{label}",
                div { "data-part": "grid", "aria-hidden": "true",
                    for (i , t) in layout.ticks.iter().enumerate() {
                        div { key: "{i}", "data-part": "gridline", style: "left: {pct(t.fraction)};" }
                    }
                }
                for (li , lane) in lanes_v.iter().enumerate() {
                    div { key: "{li}", "data-part": "lane",
                        div { "data-part": "lane-name", "{lane.name}" }
                        div { "data-part": "track", style: "--rows: {layout.lane_rows[li]};",
                            for (ii , (item , p)) in lane.items.iter().zip(placed_by_lane[li]).enumerate() {
                                div {
                                    key: "{ii}",
                                    "data-part": "item",
                                    "data-kind": if item.is_event() { "event" } else { "span" },
                                    "data-flip": (p.left > 0.6).then_some(""),
                                    role: "img",
                                    "aria-label": "{item.label}, {when(item)}",
                                    tabindex: "0",
                                    style: "left: {pct(p.left)}; width: {pct(p.width)}; --row: {p.row}; --lane: {li}; --ox-colour: var(--chart-{li % 6 + 1});",
                                    if item.is_event() {
                                        span { "data-part": "marker" }
                                    }
                                    span { "data-part": "item-label", "{item.label}" }
                                    div { "data-part": "tooltip", "aria-hidden": "true",
                                        div { "data-part": "tooltip-label", "{lane.name}" }
                                        dl {
                                            dt { "{item.label}" }
                                            dd { {when(item)} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "axis", "aria-hidden": "true",
                for (i , t) in layout.ticks.iter().enumerate() {
                    span { key: "{i}", "data-part": "tick", style: "left: {pct(t.fraction)};", "{t.label}" }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Lane" }
                        th { scope: "col", "Item" }
                        th { scope: "col", "When" }
                    }
                }
                tbody {
                    for (li , lane) in lanes_v.iter().enumerate() {
                        for (ii , item) in lane.items.iter().enumerate() {
                            tr { key: "{li}-{ii}",
                                th { scope: "row", "{lane.name}" }
                                td { "{item.label}" }
                                td { {when(item)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
