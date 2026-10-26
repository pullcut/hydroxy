//! `RadarChart`: several series over N axes. Geometry comes from `oxide_charts::radar`.
//! Every series is focusable; hovering or focusing one picks it out.

use dioxus::prelude::*;
use oxide_charts::{format_value, radar};

pub use oxide_charts::line::Series;

/// Series compared across the same axes. Axes start at zero and share one scale.
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
    /// Formats values on the rings, table and labels. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = series();
    let names = axes();
    let l = radar::layout(names.len(), &items.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), rings, max);
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    let view_box = format!("0 0 {} {}", l.width, l.height);

    rsx! {
        figure { "data-scope": "radar-chart", "data-part": "root",
            svg { "data-part": "marks", view_box: "{view_box}", role: "group", "aria-label": "{label}",
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
                for (i , (s , shape)) in items.iter().zip(l.shapes.iter()).enumerate() {
                    g {
                        key: "{i}",
                        "data-part": "series",
                        "data-index": "{i}",
                        style: "--c: var(--chart-{i % 6 + 1});",
                        tabindex: "0",
                        role: "img",
                        "aria-label": {
                            let parts: Vec<String> = names.iter().enumerate().map(|(k, n)| format!("{n} {}", value_at(s, k))).collect();
                            format!("{}: {}", s.name, parts.join(", "))
                        },
                        path { "data-part": "area", d: "{shape.path}" }
                        for (x , y) in shape.points.iter() {
                            circle { "data-part": "point", cx: "{x}", cy: "{y}", r: "3.5" }
                        }
                    }
                }
            }
            if items.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , s) in items.iter().enumerate() {
                        li { "data-part": "legend-item", "data-index": "{i}",
                            span { "data-part": "swatch", style: "background: var(--chart-{i % 6 + 1});" }
                            "{s.name}"
                        }
                    }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
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
