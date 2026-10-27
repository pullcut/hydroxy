//! `StackedBarChart`: several series stacked per category, with totals and a legend.
//! Geometry comes from `oxide_charts::stacked`; layout is plain CSS. Negative values
//! stack down from the zero line, and the total shown is the net sum.

use dioxus::prelude::*;
use oxide_charts::scale::Linear;
use oxide_charts::{annotate, bar, format_value, nav, stacked};

use crate::bar_chart::edge_side;
use crate::line_chart::annotation_nodes;
pub use oxide_charts::Orientation;
pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::line::Series;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Stacked bars: one column per label, one segment per series. Hover a column, or focus the
/// plot and use the arrow keys, to read every series in it.
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
    /// Stretch every column to 100% so the stacks compare shares. The axis and tooltips
    /// read in percent; totals and the table keep the real values.
    #[props(default)]
    normalize: bool,
    /// Reference lines, bands and labelled points. `x` values are column indices; `y`
    /// values are on the value axis (percent when normalized), which grows to fit them.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let tick = move |v: f64| if normalize { format!("{}%", format_value(v)) } else { fmt(v) };
    let horizontal = orientation == Orientation::Horizontal;
    let series_v = series();
    let labels_v = labels();
    let n = labels_v.len();
    let values: Vec<Vec<f64>> = series_v.iter().map(|s| s.values.clone()).collect();
    let drawn = if normalize { stacked::normalize(&values, n) } else { values };
    let notes = annotations();
    let layout = stacked::layout_with_extent(&drawn, n, 5, annotate::y_extent(&notes));
    let y = Linear::new((layout.min, layout.max), (0.0, 1.0));
    let placed = annotate::place(&notes, |i| bar::category_centre(i, n), |v| y.map(v));
    let placed = if horizontal { placed.into_iter().map(bar::transpose).collect() } else { placed };
    let value_at = |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
    let totals: Vec<f64> = (0..n).map(|i| series_v.iter().map(|s| value_at(s, i)).sum()).collect();
    // "4 (25%)" when normalized, else just the value.
    let reading = |si: usize, i: usize| {
        let v = series_v.get(si).map_or(0.0, |s| value_at(s, i));
        let share = drawn.get(si).and_then(|d| d.get(i)).copied().unwrap_or(0.0);
        if normalize { format!("{} ({}%)", fmt(v), format_value(share)) } else { fmt(v) }
    };
    let baseline = pct(layout.baseline);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let announcement = current.map_or_else(String::new, |i| {
        let parts: Vec<String> = series_v.iter().enumerate().map(|(si, s)| format!("{} {}", s.name, reading(si, i))).collect();
        format!("{}: {}, total {}", labels_v[i], parts.join(", "), fmt(totals[i]))
    });
    let at = |f: f64| if horizontal { format!("--p: {};", pct(f)) } else { format!("bottom: {};", pct(f)) };

    rsx! {
        figure {
            "data-scope": "stacked-bar-chart",
            "data-part": "root",
            "data-orientation": orientation.attr(),
            "data-show-totals": show_totals.then_some(""),
            "data-normalized": normalize.then_some(""),
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.value == 0.0).then_some(""),
                            style: if horizontal { format!("--p: {};", pct(g.fraction)) } else { format!("bottom: {};", pct(g.fraction)) },
                            span { "data-part": "tick", {tick(g.value)} }
                        }
                    }
                }
                div { "data-part": "annotations", "aria-hidden": "true", {annotation_nodes(&placed)} }
                div {
                    "data-part": "bars",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to read each column.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for (i, (col, name)) in layout.columns.iter().zip(labels_v.iter()).enumerate() {
                        div {
                            key: "{name}",
                            "data-part": "column",
                            "aria-hidden": "true",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--i: {i}; --ox-baseline: {baseline};",
                            onpointerenter: move |_| active.set(Some(i)),
                            div { "data-part": "stack",
                                for (si, seg) in col.segments.iter().enumerate() {
                                    if seg.height > 0.0 {
                                        div {
                                            "data-part": "segment",
                                            "data-index": "{si}",
                                            style: if horizontal {
                                                format!("--b: {}; --h: {}; background: {};", pct(seg.bottom), pct(seg.height), colour(si))
                                            } else {
                                                format!("bottom: {}; height: {}; background: {};", pct(seg.bottom), pct(seg.height), colour(si))
                                            },
                                        }
                                    }
                                }
                            }
                            span { "data-part": "total", style: at(col.top), {fmt(totals[i])} }
                            if current == Some(i) {
                                div { "data-part": "tooltip", "data-side": edge_side(i, n), style: at(col.top),
                                    div { "data-part": "tooltip-label", "{name}" }
                                    ul {
                                        for (si, ser) in series_v.iter().enumerate() {
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "swatch", style: "background: {colour(si)};" }
                                                span { "data-part": "tooltip-name", "{ser.name}" }
                                                span { "data-part": "tooltip-value", {reading(si, i)} }
                                            }
                                        }
                                    }
                                }
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
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
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
                    for (i, name) in labels_v.iter().enumerate() {
                        tr {
                            th { scope: "row", "{name}" }
                            for s in series_v.iter() {
                                td { {fmt(value_at(s, i))} }
                            }
                            td { {fmt(totals[i])} }
                        }
                    }
                }
            }
        }
    }
}
