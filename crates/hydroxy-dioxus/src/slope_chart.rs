//! `SlopeChart`: each series' change between two points in time on one shared scale.
//! Geometry comes from `hydroxy_charts::rank`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, rank};

use crate::plot_kit::{ChartTip, colour};
use crate::rank_kit::{Dim, SeriesLegend, series_style};

pub use hydroxy_charts::line::Series;

fn pair(s: &Series) -> (f64, f64) {
    (s.values.first().copied().unwrap_or(f64::NAN), s.values.get(1).copied().unwrap_or(f64::NAN))
}

/// Two columns of values joined by a line per series. Each series needs two values: before and after.
/// Hover or touch a line or end to read it; focus the chart and use the arrow keys
/// (Left/Right for the end, Up/Down for the series), Home and End to jump, Escape to clear.
/// Hovering a legend item picks its series out.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SlopeChart(
    /// One series per line; `values` is `[from, to]`.
    series: ReadSignal<Vec<Series>>,
    /// Heading over the left axis, such as "2023".
    from_label: String,
    /// Heading over the right axis, such as "2024".
    to_label: String,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Formats values in the labels, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = series();
    let n = items.len();
    let l = rank::slope(&items.iter().map(pair).collect::<Vec<_>>());
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let (left, right) = (l.x.0 - 12.0, l.x.1 + 12.0);
    // (series, end): end 0 is "from", 1 is "to".
    let mut active = use_signal(|| None::<(usize, usize)>);
    let dim = Dim::new(active);
    let current = active().filter(|&(s, e)| s < n && e < 2);
    let now = current.and_then(|(si, end)| EventDatum::from_series(&items, si, end, Some(items[si].name.as_str())));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let announcement = current.map(|(si, _)| {
        let (a, b) = pair(&items[si]);
        format!("{}: {from_label} {}, {to_label} {}", items[si].name, fmt(a), fmt(b))
    });
    let tip = current.map(|(si, end)| {
        let line = l.lines[si];
        let (x, y) = if end == 0 { (l.x.0, line.y.0) } else { (l.x.1, line.y.1) };
        let (from, to) = pair(&items[si]);
        (si, end, x / l.width, 1.0 - y / l.height, from, to)
    });

    rsx! {
        figure { "data-scope": "slope-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the lines.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| crate::callbacks::select_active(Some(datum), on_select),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::grid_key(&e.key().to_string(), active(), n, 2) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { "data-part": "marks", view_box: "{view_box}", "aria-hidden": "true",
                    text { "data-part": "column-label", x: "{l.x.0}", y: "14", "text-anchor": "middle", "{from_label}" }
                    text { "data-part": "column-label", x: "{l.x.1}", y: "14", "text-anchor": "middle", "{to_label}" }
                    for (i , (s , line)) in items.iter().zip(l.lines.iter()).enumerate() {
                        {
                            let (a, b) = pair(s);
                            let dir = if b > a { "up" } else if b < a { "down" } else { "flat" };
                            rsx! {
                                g {
                                    key: "{i}",
                                    "data-part": "series",
                                    "data-index": "{i}",
                                    "data-direction": dir,
                                    "data-dim": dim.of(i),
                                    style: series_style(i),
                                    onpointerenter: move |_| {
                                        if active().map(|(s, _)| s) != Some(i) {
                                            active.set(Some((i, 1)));
                                        }
                                    },
                                    line { "data-part": "line", x1: "{l.x.0}", y1: "{line.y.0}", x2: "{l.x.1}", y2: "{line.y.1}" }
                                    circle { "data-part": "point", "data-active": (current == Some((i, 0))).then_some(""), cx: "{l.x.0}", cy: "{line.y.0}", r: "4.5" }
                                    circle { "data-part": "point", "data-active": (current == Some((i, 1))).then_some(""), cx: "{l.x.1}", cy: "{line.y.1}", r: "4.5" }
                                    circle {
                                        "data-part": "hit",
                                        cx: "{l.x.0}",
                                        cy: "{line.y.0}",
                                        r: "14",
                                        onpointerenter: move |_| active.set(Some((i, 0))),
                                        onpointerdown: move |_| active.set(Some((i, 0))),
                                    }
                                    circle {
                                        "data-part": "hit",
                                        cx: "{l.x.1}",
                                        cy: "{line.y.1}",
                                        r: "14",
                                        onpointerenter: move |_| active.set(Some((i, 1))),
                                        onpointerdown: move |_| active.set(Some((i, 1))),
                                    }
                                    text { "data-part": "name", x: "{left}", y: "{line.label_y.0}", "text-anchor": "end", "{s.name}  {fmt(a)}" }
                                    text { "data-part": "name", x: "{right}", y: "{line.label_y.1}", "text-anchor": "start", "{fmt(b)}  {s.name}" }
                                }
                            }
                        }
                    }
                }
                if let Some((si, end, x, y, from, to)) = tip {
                    ChartTip {
                        tip: Tip {
                            x,
                            y,
                            title: items[si].name.clone(),
                            rows: vec![
                                TipRow::new(None, from_label.clone(), fmt(from)),
                                TipRow::new(None, to_label.clone(), fmt(to)),
                                TipRow::new(None, "Change", format!("{}{}", if to - from > 0.0 { "+" } else { "" }, fmt(to - from))),
                            ],
                        },
                        swatch: colour(si),
                        anchored: true,
                        side: Some(if end == 0 { "right" } else { "left" }),
                    }
                }
            }
            SeriesLegend { series, dim }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Series" }
                        th { scope: "col", "{from_label}" }
                        th { scope: "col", "{to_label}" }
                    }
                }
                tbody {
                    for s in items.iter() {
                        tr {
                            th { scope: "row", "{s.name}" }
                            td { {fmt(pair(s).0)} }
                            td { {fmt(pair(s).1)} }
                        }
                    }
                }
            }
        }
    }
}
