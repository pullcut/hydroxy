//! `TimelineChart`: spans and events in lanes on a date axis. Placement and calendar
//! ticks come from `hydroxy_charts::timeline`; layout is plain CSS percentages.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{nav, timeline};

use crate::callbacks::{select_active, use_hover};
use crate::plot_kit::{colour, pct};
use crate::sparkline::use_datum;

pub use hydroxy_charts::timeline::{DAY, HOUR, Item, Lane, ms_from_civil};

/// Time ranges and moments, one row group per lane. Overlapping items in a lane stack.
/// Times are milliseconds since the Unix epoch (UTC); see [`ms_from_civil`].
/// Hover or touch an item to read its dates; focus the chart and use the arrow keys to
/// step through items lane by lane, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TimelineChart(
    /// The lanes, top to bottom.
    lanes: ReadSignal<Vec<Lane>>,
    /// What the chart shows, such as "Release plan". Used as the accessible name.
    label: String,
    /// Formats times for the tooltip, live region and table. Default: "5 Oct 2026", with
    /// the time when it is not midnight.
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused item changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on an item.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| timeline::format_date(v), |f| f.call(v));
    let when = move |i: &Item| if i.is_event() { fmt(i.start) } else { format!("{} to {}", fmt(i.start), fmt(i.end)) };
    let lanes_v = lanes();
    let layout = timeline::layout_lanes(&lanes_v, 6);
    let mut offset = 0;
    let mut placed_by_lane = Vec::new();
    for lane in &lanes_v {
        placed_by_lane.push(&layout.items[offset..offset + lane.items.len()]);
        offset += lane.items.len();
    }

    let count = layout.items.len();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < count);
    let announcement = current.and_then(|at| {
        let (lane, item) = lanes_v.iter().flat_map(|lane| lane.items.iter().map(move |i| (lane, i))).nth(at)?;
        Some(format!("{}: {}, {}", lane.name, item.label, when(item)))
    });
    let starts: Vec<usize> = lanes_v
        .iter()
        .scan(0, |acc, lane| {
            let s = *acc;
            *acc += lane.items.len();
            Some(s)
        })
        .collect();

    // `value` is the item's start time in ms since the epoch; `series` is its lane.
    let datum = use_datum(current.and_then(|at| {
        let li = starts.iter().rposition(|&s| s <= at)?;
        let index = at - starts[li];
        let item = lanes_v[li].items.get(index)?;
        Some(EventDatum { series: li, index, label: Some(item.label.clone()), value: item.start, node: None })
    }));
    use_hover(datum, on_hover);

    rsx! {
        figure { "data-scope": "timeline-chart", "data-part": "root",
            div {
                "data-part": "body",
                role: "group",
                tabindex: "0",
                "aria-label": "{label}. Use the arrow keys to step through items.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| select_active(datum, on_select),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        select_active(datum, on_select);
                    }
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), count) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
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
                                    "data-active": (current == Some(starts[li] + ii)).then_some(""),
                                    onpointerenter: {
                                        let at = starts[li] + ii;
                                        move |_| active.set(Some(at))
                                    },
                                    onpointerdown: {
                                        let at = starts[li] + ii;
                                        move |_| active.set(Some(at))
                                    },
                                    style: "left: {pct(p.left)}; width: {pct(p.width)}; --row: {p.row}; --lane: {li}; --ox-colour: {colour(li)};",
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
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
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
