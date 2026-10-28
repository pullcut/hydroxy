//! `RadarChart`: several series over N axes. Geometry comes from `hydroxy_charts::radar`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, radar};

use crate::plot_kit::{ChartTip, colour};
use crate::rank_kit::{Dim, SeriesLegend, series_style};

pub use hydroxy_charts::line::Series;

/// Series compared across the same axes. Axes start at zero and share one scale.
/// Hover or touch a vertex to read it; focus the chart and use the arrow keys
/// (Left/Right around the axes, Up/Down between series), Home and End to jump, Escape
/// to clear. Hovering a legend item picks its series out.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RadarChart(
    /// The series. `values` has one entry per axis. Up to four stay legible.
    series: ReadSignal<Vec<Series>>,
    /// One name per axis, clockwise from the top. Three or more.
    axes: ReadSignal<Vec<String>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Value at the outer ring. Default: the data maximum rounded up to a nice number.
    max: Option<f64>,
    /// Number of guide rings.
    #[props(default = 4)]
    rings: usize,
    /// Formats values on the rings, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = series();
    let names = axes();
    let l = radar::layout(names.len(), &items.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), rings, max);
    let value_at = move |s: &Series, i: usize| {
        s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let (rows, cols) = (items.len(), names.len());
    // (series, axis)
    let mut active = use_signal(|| None::<(usize, usize)>);
    let dim = Dim::new(active);
    let current = active().filter(|&(s, k)| s < rows && k < cols);
    let now = current.and_then(|(si, k)| EventDatum::from_series(&items, si, k, Some(&names[k])));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let announcement = current.map(|(si, k)| format!("{}, {}: {}", items[si].name, names[k], value_at(&items[si], k)));
    let tip = current.map(|(si, k)| {
        let (x, y) = l.shapes[si].points[k];
        (si, k, x < l.centre.0, x / l.width, 1.0 - y / l.height)
    });

    rsx! {
        figure { "data-scope": "radar-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the values.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| crate::callbacks::select_active(Some(datum), on_select),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::grid_key(&e.key().to_string(), active(), rows, cols) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { "data-part": "marks", view_box: "{view_box}", "aria-hidden": "true",
                    for (v , d) in l.rings.iter() {
                        path { "data-part": "ring", d: "{d}" }
                        text {
                            "data-part": "tick",
                            x: "{l.centre.0 + 4.0}",
                            y: "{l.centre.1 - 130.0 * v / l.max + 10.0}",
                            {fmt(*v)}
                        }
                    }
                    for (a , n) in l.axes.iter().zip(names.iter()) {
                        line { "data-part": "spoke", x1: "{l.centre.0}", y1: "{l.centre.1}", x2: "{a.end.0}", y2: "{a.end.1}" }
                        text { "data-part": "axis-label", x: "{a.label.0}", y: "{a.label.1}", "text-anchor": a.anchor, "{n}" }
                    }
                    for (i , shape) in l.shapes.iter().enumerate() {
                        g {
                            key: "{i}",
                            "data-part": "series",
                            "data-index": "{i}",
                            "data-dim": dim.of(i),
                            style: series_style(i),
                            path { "data-part": "area", d: "{shape.path}" }
                            for (k , (x , y)) in shape.points.iter().enumerate() {
                                circle { "data-part": "point", "data-active": (current == Some((i, k))).then_some(""), cx: "{x}", cy: "{y}", r: "3.5" }
                                circle {
                                    "data-part": "hit",
                                    cx: "{x}",
                                    cy: "{y}",
                                    r: "12",
                                    onpointerenter: move |_| active.set(Some((i, k))),
                                    onpointerdown: move |_| active.set(Some((i, k))),
                                }
                            }
                        }
                    }
                }
                if let Some((si, k, flip, x, y)) = tip {
                    ChartTip {
                        tip: Tip {
                            x,
                            y,
                            title: items[si].name.clone(),
                            rows: vec![TipRow::new(None, names[k].clone(), value_at(&items[si], k))],
                        },
                        swatch: colour(si),
                        anchored: true,
                        side: Some(if flip { "left" } else { "right" }),
                    }
                }
            }
            SeriesLegend { series, dim }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Axis" }
                        for s in items.iter() {
                            th { scope: "col", "{s.name}" }
                        }
                    }
                }
                tbody {
                    for (i , a) in names.iter().enumerate() {
                        tr {
                            th { scope: "row", "{a}" }
                            for s in items.iter() {
                                td { {value_at(s, i)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
