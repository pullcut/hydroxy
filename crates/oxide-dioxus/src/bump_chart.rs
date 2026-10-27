//! `BumpChart`: how each series ranks at every step. Geometry comes from
//! `oxide_charts::rank`.

use dioxus::prelude::*;
use oxide_charts::{format_value, nav, rank};

pub use oxide_charts::line::Series;

/// Rank over time. Values decide the order at each column; the best rank is on top.
/// Hover or touch a marker to read it; focus the chart and use the arrow keys
/// (Left/Right along a line, Up/Down between lines), Home and End to jump, Escape to clear.
/// Hovering a legend item picks its series out.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
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
    /// Formats values in the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = series();
    let heads = labels();
    let l = rank::bump(&items.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), higher_is_better);
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let (left, right) = (l.edges.0 - 14.0, l.edges.1 + 14.0);
    let (rows, cols) = (l.lines.len(), l.columns.len());
    let mut active = use_signal(|| None::<(usize, usize)>);
    let mut legend_hover = use_signal(|| None::<usize>);
    let current = active().filter(|&(s, c)| s < rows && c < cols);
    let focus = current.map(|(s, _)| s).or(legend_hover());
    let dim = |i: usize| focus.is_some_and(|f| f != i).then_some("");
    let column = |ci: usize| heads.get(ci).cloned().unwrap_or_default();
    let announcement =
        current.map(|(si, ci)| format!("{}, {}: rank {} ({})", items[si].name, column(ci), l.lines[si].ranks[ci] + 1, value_at(&items[si], ci)));
    let tip = current.map(|(si, ci)| {
        let (x, y) = l.lines[si].points[ci];
        (si, ci, x / l.width, y / l.height)
    });

    rsx! {
        figure { "data-scope": "bump-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the ranks.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::grid_key(&e.key().to_string(), active(), rows, cols) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { "data-part": "marks", view_box: "{view_box}", "aria-hidden": "true",
                    for (t , x) in heads.iter().zip(l.columns.iter()) {
                        text { "data-part": "column-label", x: "{x}", y: "14", "text-anchor": "middle", "{t}" }
                    }
                    for (si , line) in l.lines.iter().enumerate() {
                        g {
                            key: "{si}",
                            "data-part": "series",
                            "data-index": "{si}",
                            "data-dim": dim(si),
                            style: "--c: var(--chart-{si % 6 + 1});",
                            path { "data-part": "line", d: "{line.path}" }
                            for (ci , (x , y)) in line.points.iter().enumerate() {
                                circle { "data-part": "point", "data-active": (current == Some((si, ci))).then_some(""), cx: "{x}", cy: "{y}", r: "5" }
                                circle {
                                    "data-part": "hit",
                                    cx: "{x}",
                                    cy: "{y}",
                                    r: "16",
                                    onpointerenter: move |_| active.set(Some((si, ci))),
                                    onpointerdown: move |_| active.set(Some((si, ci))),
                                }
                            }
                            if let (Some(first), Some(last)) = (line.points.first(), line.points.last()) {
                                text { "data-part": "name", x: "{left}", y: "{first.1}", "text-anchor": "end", "{items[si].name}" }
                                text { "data-part": "name", x: "{right}", y: "{last.1}", "text-anchor": "start", "{items[si].name}" }
                            }
                        }
                    }
                }
                if let Some((si, ci, fx, fy)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if fx > 0.5 { "left" } else { "right" },
                        style: "left: {fx * 100.0:.3}%; top: {fy * 100.0:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: var(--chart-{si % 6 + 1});" }
                            "{items[si].name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", {column(ci)} }
                                span { "data-part": "tooltip-value", "#{l.lines[si].ranks[ci] + 1}" }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {value_at(&items[si], ci)} }
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
                        th { scope: "col", "Series" }
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
                                td {
                                    {
                                        l.lines[si]
                                            .ranks
                                            .get(ci)
                                            .map_or_else(|| "–".to_string(), |r| format!("{} ({})", r + 1, value_at(s, ci)))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
