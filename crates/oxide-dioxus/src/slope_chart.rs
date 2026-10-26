//! `SlopeChart`: each series' change between two points in time on one shared scale.
//! Geometry comes from `oxide_charts::rank`. Every line is focusable.

use dioxus::prelude::*;
use oxide_charts::{format_value, rank};

pub use oxide_charts::line::Series;

/// Two columns of values joined by a line per series. Each series needs two values: before and after.
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
    /// Formats values in the labels and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let pair = |s: &Series| (s.values.first().copied().unwrap_or(f64::NAN), s.values.get(1).copied().unwrap_or(f64::NAN));
    let items = series();
    let l = rank::slope(&items.iter().map(pair).collect::<Vec<_>>());
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let (left, right) = (l.x.0 - 12.0, l.x.1 + 12.0);

    rsx! {
        figure { "data-scope": "slope-chart", "data-part": "root",
            svg { "data-part": "marks", view_box: "{view_box}", role: "group", "aria-label": "{label}",
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
                                style: "--c: var(--chart-{i % 6 + 1});",
                                tabindex: "0",
                                role: "img",
                                "aria-label": "{s.name}: {from_label} {fmt(a)} to {to_label} {fmt(b)}",
                                line { "data-part": "line", x1: "{l.x.0}", y1: "{line.y.0}", x2: "{l.x.1}", y2: "{line.y.1}" }
                                circle { "data-part": "point", cx: "{l.x.0}", cy: "{line.y.0}", r: "4.5" }
                                circle { "data-part": "point", cx: "{l.x.1}", cy: "{line.y.1}", r: "4.5" }
                                text { "data-part": "name", x: "{left}", y: "{line.label_y.0}", "text-anchor": "end", "{s.name}  {fmt(a)}" }
                                text { "data-part": "name", x: "{right}", y: "{line.label_y.1}", "text-anchor": "start", "{fmt(b)}  {s.name}" }
                            }
                        }
                    }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
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
