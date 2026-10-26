//! `StackedBarChart`: several series stacked per category, with totals and a legend.
//! Geometry comes from `oxide_charts::stacked`; layout is plain CSS. Negative values
//! stack down from the zero line, and the total shown is the net sum.

use dioxus::prelude::*;
use oxide_charts::{format_value, stacked};

pub use oxide_charts::Orientation;
pub use oxide_charts::line::Series;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Vertical stacked bars: one column per label, one segment per series.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn StackedBarChart(
    /// The series, stacked bottom-up in order, in categorical colours. Up to six read well.
    series: ReadSignal<Vec<Series>>,
    /// One label per column.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Tickets by channel per week". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the labels and legend.
    #[props(default = 200)]
    height: u16,
    /// Vertical (default) or horizontal stacks. Horizontal suits long category names.
    #[props(default)]
    orientation: Orientation,
    /// Show each column's total above it.
    #[props(default = true)]
    show_totals: bool,
    /// Formats values for the axis, totals and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let horizontal = orientation == Orientation::Horizontal;
    let series_v = series();
    let labels_v = labels();
    let values: Vec<Vec<f64>> = series_v.iter().map(|s| s.values.clone()).collect();
    let layout = stacked::layout(&values, labels_v.len(), 5);
    let value_at = |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
    let baseline = pct(layout.baseline);

    rsx! {
        figure { "data-scope": "stacked-bar-chart", "data-part": "root", "data-orientation": orientation.attr(), "data-show-totals": show_totals.then_some(""),
            div { "data-part": "plot", style: "height: {height}px;", "aria-hidden": "true",
                div { "data-part": "grid",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.value == 0.0).then_some(""),
                            style: if horizontal { format!("--p: {};", pct(g.fraction)) } else { format!("bottom: {};", pct(g.fraction)) },
                            span { "data-part": "tick", {fmt(g.value)} }
                        }
                    }
                }
                div { "data-part": "bars",
                    for (i, (col, name)) in layout.columns.iter().zip(labels_v.iter()).enumerate() {
                        div { key: "{name}", "data-part": "column", style: "--i: {i}; --ox-baseline: {baseline};",
                            div { "data-part": "stack",
                                for (si, (seg, ser)) in col.segments.iter().zip(series_v.iter()).enumerate() {
                                    if seg.height > 0.0 {
                                        div {
                                            "data-part": "segment",
                                            "data-index": "{si}",
                                            title: "{name}, {ser.name}: {fmt(value_at(ser, i))}",
                                            style: if horizontal {
                                                format!("--b: {}; --h: {}; background: {};", pct(seg.bottom), pct(seg.height), colour(si))
                                            } else {
                                                format!("bottom: {}; height: {}; background: {};", pct(seg.bottom), pct(seg.height), colour(si))
                                            },
                                        }
                                    }
                                }
                            }
                            span {
                                "data-part": "total",
                                style: if horizontal { format!("--p: {};", pct(col.top)) } else { format!("bottom: {};", pct(col.top)) },
                                {fmt(col.total)}
                            }
                        }
                    }
                }
            }
            div { "data-part": "labels", "aria-hidden": "true",
                for l in labels_v.iter() {
                    span { "data-part": "label", title: "{l}", "{l}" }
                }
            }
            ul { "data-part": "legend", "aria-hidden": "true",
                for (i, s) in series_v.iter().enumerate() {
                    li { "data-part": "legend-item", "data-index": "{i}",
                        span { "data-part": "swatch", style: "background: {colour(i)};" }
                        "{s.name}"
                    }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
                        for s in series_v.iter() {
                            th { scope: "col", "{s.name}" }
                        }
                        th { scope: "col", "Total" }
                    }
                }
                tbody {
                    for (i, (name, col)) in labels_v.iter().zip(layout.columns.iter()).enumerate() {
                        tr {
                            th { scope: "row", "{name}" }
                            for s in series_v.iter() {
                                td { {fmt(value_at(s, i))} }
                            }
                            td { {fmt(col.total)} }
                        }
                    }
                }
            }
        }
    }
}
