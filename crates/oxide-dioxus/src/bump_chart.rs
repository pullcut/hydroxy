//! `BumpChart`: how each series ranks at every step. Geometry comes from
//! `oxide_charts::rank`. Every line is focusable; hovering or focusing one picks it out.

use dioxus::prelude::*;
use oxide_charts::{format_value, rank};

pub use oxide_charts::line::Series;

/// Rank over time. Values decide the order at each column; the best rank is on top.
#[component]
pub fn BumpChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    series: ReadSignal<Vec<Series>>,
    /// One label per column, such as a season or month.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Rank the highest value first (default). Turn off for measures like lap time.
    #[props(default = true)]
    higher_is_better: bool,
    /// Formats values in the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = series();
    let heads = labels();
    let l = rank::bump(&items.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), higher_is_better);
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let (left, right) = (l.edges.0 - 14.0, l.edges.1 + 14.0);

    rsx! {
        figure { "data-scope": "bump-chart", "data-part": "root",
            svg { "data-part": "marks", view_box: "{view_box}", role: "group", "aria-label": "{label}",
                for (t , x) in heads.iter().zip(l.columns.iter()) {
                    text { "data-part": "column-label", x: "{x}", y: "14", "text-anchor": "middle", "{t}" }
                }
                for (si , line) in l.lines.iter().enumerate() {
                    g {
                        key: "{si}",
                        "data-part": "series",
                        "data-index": "{si}",
                        style: "--c: var(--chart-{si % 6 + 1});",
                        tabindex: "0",
                        role: "img",
                        "aria-label": {
                            let parts: Vec<String> = (0..line.ranks.len())
                                .map(|ci| format!("{}, {}: rank {} ({})", items[si].name, heads.get(ci).cloned().unwrap_or_default(), line.ranks[ci] + 1, value_at(&items[si], ci)))
                                .collect();
                            parts.join("; ")
                        },
                        path { "data-part": "line", d: "{line.path}" }
                        for (x , y) in line.points.iter() {
                            circle { "data-part": "point", cx: "{x}", cy: "{y}", r: "5" }
                        }
                        if let (Some(first), Some(last)) = (line.points.first(), line.points.last()) {
                            text { "data-part": "name", x: "{left}", y: "{first.1}", "text-anchor": "end", "{items[si].name}" }
                            text { "data-part": "name", x: "{right}", y: "{last.1}", "text-anchor": "start", "{items[si].name}" }
                        }
                    }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
                        for t in heads.iter() {
                            th { scope: "col", "{t}" }
                        }
                    }
                }
                tbody {
                    for (si , s) in items.iter().enumerate() {
                        tr {
                            th { scope: "row", "{s.name}" }
                            for ci in 0..heads.len() {
                                td { "{l.lines[si].ranks[ci] + 1} ({value_at(s, ci)})" }
                            }
                        }
                    }
                }
            }
        }
    }
}
