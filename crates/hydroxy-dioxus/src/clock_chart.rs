//! `ClockChart`: activity by hour of the day as radial bars around a 24-hour clock face,
//! midnight at the top. Geometry comes from `hydroxy_charts::clock`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, colour};
use hydroxy_charts::{clock, format_value, funnel, polar};

use crate::plot_kit::pct;

pub use hydroxy_charts::line::Series;

/// "09:00–10:00" for slot `i` of `n` around the day.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "slot counts are tiny"
)]
fn slot_name(i: usize, n: usize) -> String {
    let minutes = |k: usize| (k as f64 * 1440.0 / n.max(1) as f64).round() as usize;
    let (a, b) = (minutes(i), minutes(i + 1));
    format!("{:02}:{:02}–{:02}:{:02}", a / 60, a % 60, b / 60 % 24, b % 60)
}

/// Angle of the start of slot `h` of `n`, radians clockwise from 12.
#[allow(clippy::cast_precision_loss, reason = "slot counts are tiny")]
fn start(h: usize, n: usize) -> f64 {
    h as f64 / n.max(1) as f64 * std::f64::consts::TAU
}

/// A 24-hour clock face with a bar per hour growing out from the centre, so the shape of a
/// day (a morning peak, a quiet night) reads at a glance. A second series sits beside the
/// first in each hour, for comparing two days or two groups. Hover or touch an hour to
/// read it; focus the chart and use the arrow keys to step through the hours, Home and
/// End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ClockChart(
    /// One or two series with a value per hour from midnight (24 values; other counts split the day evenly).
    series: ReadSignal<Vec<Series>>,
    /// What the chart shows, such as "Support chats by hour". Used as the accessible name.
    label: String,
    /// Face diameter in px.
    #[props(default = 300)]
    size: u16,
    /// Formats values for the rings, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let all: Vec<Series> = series.read().iter().take(2).cloned().collect();
    let layout = clock::layout(&all.iter().map(|s| s.values.as_slice()).collect::<Vec<_>>());
    let n = layout.slots;
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.and_then(|i| EventDatum::from_series(&all, 0, i, Some(slot_name(i, n).as_str())))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let announcement = current
        .map(|i| {
            let values: Vec<String> =
                all.iter().map(|s| format!("{} {}", s.name, plot::cell(s.values.get(i).copied(), &fmt))).collect();
            format!("{}: {}", slot_name(i, n), values.join(", "))
        })
        .unwrap_or_default();
    let width = format!("{:.3}", layout.width);
    let step = if n == 24 { 3 } else { n.div_ceil(8).max(1) };

    rsx! {
        figure { "data-scope": "clock-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the hours.",
                style: "width: {size}px;",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    let key = e.key().to_string();
                    if let Some(next) = list_key(funnel::arrow_alias(&key), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 100 100", "aria-hidden": "true",
                    circle { "data-part": "face", cx: "50", cy: "50", r: "{clock::R:.3}" }
                    for r in layout.rings.iter() {
                        circle { "data-part": "ring", cx: "50", cy: "50", r: "{r.radius:.3}" }
                    }
                    circle { "data-part": "baseline", cx: "50", cy: "50", r: "{clock::INNER:.3}" }
                    for h in 0..n {
                        {
                            let major = n == 24 && h % 6 == 0;
                            let (x0, y0) = polar::at(if major { 0.9 } else { 0.94 }, start(h, n));
                            let (x1, y1) = polar::at(1.0, start(h, n));
                            rsx! {
                                line {
                                    "data-part": "hour-tick",
                                    "data-major": major.then_some(""),
                                    x1: "{x0 * 100.0:.3}",
                                    y1: "{y0 * 100.0:.3}",
                                    x2: "{x1 * 100.0:.3}",
                                    y2: "{y1 * 100.0:.3}",
                                }
                            }
                        }
                    }
                    for b in layout.bars.iter() {
                        path {
                            key: "{b.slot}-{b.series}",
                            "data-part": "bar",
                            "data-index": "{b.series}",
                            "data-empty": b.value.is_none().then_some(""),
                            "data-active": (current == Some(b.slot)).then_some(""),
                            "pathLength": "1",
                            stroke: colour(b.series),
                            stroke_width: "{width}",
                            style: "--i: {b.slot};",
                            d: "{b.path}",
                        }
                    }
                    for (i , d) in layout.hits.iter().enumerate() {
                        path {
                            "data-part": "hit",
                            d: "{d}",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: move |_| {
                                active.set(Some(i));
                                crate::callbacks::select_active(Some(datum), on_select);
                            },
                        }
                    }
                }
                for h in (0..n).step_by(step) {
                    {
                        let (x, y) = polar::at(1.1, start(h, n));
                        rsx! {
                            span { "data-part": "hour", "aria-hidden": "true", style: "left: {pct(x)}; top: {pct(y)};", {slot_name(h, n)[..2].to_string()} }
                        }
                    }
                }
                for r in layout.rings.iter() {
                    span { "data-part": "ring-label", "aria-hidden": "true", style: "top: {pct(0.5 - r.radius / 100.0)};", {fmt(r.value)} }
                }
                if let Some(i) = current {
                    {
                        let (x, y) = clock::anchor(i, n);
                        rsx! {
                            div {
                                "data-part": "tooltip",
                                "data-side": if x > 0.5 { "left" } else { "right" },
                                style: "left: {pct(x)}; top: {pct(y)};",
                                "aria-hidden": "true",
                                div { "data-part": "tooltip-label", {slot_name(i, n)} }
                                ul {
                                    for (k , s) in all.iter().enumerate() {
                                        li { "data-part": "tooltip-row",
                                            span { "data-part": "tooltip-name",
                                                span { "data-part": "swatch", style: "background: {colour(k)};" }
                                                "{s.name}"
                                            }
                                            span { "data-part": "tooltip-value", {plot::cell(s.values.get(i).copied(), &fmt)} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if all.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (k , s) in all.iter().enumerate() {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {colour(k)};" }
                            "{s.name}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Hour" }
                        for s in all.iter() {
                            th { scope: "col", "{s.name}" }
                        }
                    }
                }
                tbody {
                    for i in 0..n {
                        tr {
                            th { scope: "row", {slot_name(i, n)} }
                            for s in all.iter() {
                                td { {plot::cell(s.values.get(i).copied(), &fmt)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
