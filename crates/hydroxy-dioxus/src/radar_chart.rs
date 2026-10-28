//! `RadarChart`: several series over N axes. Geometry comes from `hydroxy_charts::radar`.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, nav, radar};

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
    let mut legend_hover = use_signal(|| None::<usize>);
    let current = active().filter(|&(s, k)| s < rows && k < cols);
    let focus = current.map(|(s, _)| s).or(legend_hover());
    let dim = |i: usize| focus.is_some_and(|f| f != i).then_some("");
    let announcement = current.map(|(si, k)| format!("{}, {}: {}", items[si].name, names[k], value_at(&items[si], k)));
    let tip = current.map(|(si, k)| {
        let (x, y) = l.shapes[si].points[k];
        (si, k, x < l.centre.0, x / l.width * 100.0, y / l.height * 100.0)
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
                onkeydown: move |e| {
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
                            "data-dim": dim(i),
                            style: "--c: var(--chart-{i % 6 + 1});",
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
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if flip { "left" } else { "right" },
                        style: "left: {x:.3}%; top: {y:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: var(--chart-{si % 6 + 1});" }
                            "{items[si].name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "{names[k]}" }
                                span { "data-part": "tooltip-value", {value_at(&items[si], k)} }
                            }
                        }
                    }
                }
            }
            if items.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , s) in items.iter().enumerate() {
                        li {
                            "data-part": "legend-item",
                            "data-index": "{i}",
                            "data-dim": dim(i),
                            onpointerenter: move |_| legend_hover.set(Some(i)),
                            onpointerleave: move |_| legend_hover.set(None),
                            span { "data-part": "swatch", style: "background: var(--chart-{i % 6 + 1});" }
                            "{s.name}"
                        }
                    }
                }
            }
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
