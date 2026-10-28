//! `CalendarHeatmapChart`: one cell per day in week columns, shaded by value, like a
//! contribution graph. Spans over a year stack one row per calendar year. Geometry comes
//! from `oxide_charts::calendar`.

use dioxus::prelude::*;
use oxide_charts::calendar::{self, WEEKDAYS};
use oxide_charts::scale::days_from_civil;
use oxide_charts::{format_value, raster};

use crate::chart_frame::use_export;

/// A day-by-day calendar shaded from the lowest value (or zero) to the highest in one
/// chart colour. Hover or touch a day to read it; focus the calendar and use Up/Down to
/// step a day, Left/Right a week, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn CalendarHeatmapChart(
    /// One value per day, from `start`. Non-finite values draw as empty days.
    values: ReadSignal<Vec<f64>>,
    /// The first day, as (year, month 1..=12, day).
    start: ReadSignal<(i64, u32, u32)>,
    /// What the chart shows, such as "Commits per day". Used as the accessible name.
    label: String,
    /// Formats values for the tooltip, legend and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let layout = use_memo(move || {
        let (y, m, d) = start();
        calendar::layout(days_from_civil(y, m, d), &values.read())
    });
    let mut active = use_signal(|| None::<usize>);
    use_export(move |()| {
        let l = layout.read();
        std::iter::once(vec!["Date".to_string(), "Value".to_string()])
            .chain(l.days.iter().map(|d| vec![calendar::day_name(d.date), d.value.map_or_else(String::new, fmt)]))
            .collect()
    });
    let l = layout.read();
    let len = l.days.len();
    let current = active().filter(|&i| i < len);
    let value_text = |v: Option<f64>| v.map_or_else(|| "No data".to_string(), fmt);
    let announcement = current.map(|i| format!("{}: {}", calendar::day_name(l.days[i].date), value_text(l.days[i].value))).unwrap_or_default();
    let shown = raster::table_rows(len, 2);

    rsx! {
        figure { "data-scope": "calendar-heatmap-chart", "data-part": "root", style: "--weeks: {l.weeks.max(1)};",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to move between days.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = calendar::key(&e.key().to_string(), active(), len) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                for b in l.blocks.iter() {
                    div { "data-part": "block",
                        if !b.label.is_empty() {
                            span { "data-part": "year", "{b.label}" }
                        }
                        div { "data-part": "months", "aria-hidden": "true",
                            for m in b.months.iter() {
                                span { "data-part": "month", style: "--col: {m.week + 1};", "{m.name}" }
                            }
                        }
                        div { "data-part": "weekdays", "aria-hidden": "true",
                            for (r , w) in WEEKDAYS.iter().enumerate() {
                                span { "data-part": "weekday", if r % 2 == 0 { "{w}" } }
                            }
                        }
                        div { "data-part": "cells",
                            for (i , d) in l.days.iter().enumerate().take(b.end).skip(b.first) {
                                span {
                                    key: "{d.date}",
                                    "data-part": "cell",
                                    "data-empty": d.value.is_none().then_some(""),
                                    "data-active": (current == Some(i)).then_some(""),
                                    "data-edge": calendar::edge(d.week, l.weeks),
                                    style: "--col: {d.week + 1}; --row: {d.weekday + 1}; --level: {d.level:.3}; --i: {d.week};",
                                    onpointerenter: move |_| active.set(Some(i)),
                                    onpointerdown: move |_| active.set(Some(i)),
                                    if current == Some(i) {
                                        span { "data-part": "tooltip", "aria-hidden": "true",
                                            span { "data-part": "tooltip-label", {calendar::day_name(d.date)} }
                                            span { "data-part": "tooltip-value", {value_text(d.value)} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "scale", "aria-hidden": "true",
                span { "data-part": "scale-value", {fmt(l.min)} }
                span { "data-part": "ramp" }
                span { "data-part": "scale-value", {fmt(l.max)} }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption {
                    "{label}"
                    if shown < len {
                        " (first {shown} of {len} days)"
                    }
                }
                thead {
                    tr {
                        th { scope: "col", "Date" }
                        th { scope: "col", "Value" }
                    }
                }
                tbody {
                    for d in l.days.iter().take(shown) {
                        tr {
                            th { scope: "row", {calendar::day_name(d.date)} }
                            td { {d.value.map_or_else(|| "–".to_string(), fmt)} }
                        }
                    }
                }
            }
        }
    }
}
