//! `HeatmapChart`: a rows × columns grid whose cells shade by value. The scale is one
//! chart colour mixed into the surface, so it follows the theme. Geometry comes from
//! `oxide_charts::heatmap`.

use dioxus::prelude::*;
use oxide_charts::{format_value, heatmap};

#[allow(clippy::cast_precision_loss, reason = "grid sizes are tiny")]
fn tip_style(r: usize, c: usize, rows: usize, cols: usize) -> String {
    let below = if r == 0 { 1.0 } else { 0.0 };
    format!("left: {:.3}%; top: {:.3}%;", (c as f64 + 0.5) / cols as f64 * 100.0, (r as f64 + below) / rows as f64 * 100.0)
}

/// A grid of values, one `Vec` per row, shaded from the lowest to the highest value.
/// Hover a cell to read it; focus the grid and use the arrow keys, Home and End to move,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn HeatmapChart(
    /// Values by row, then column. Missing or non-finite entries draw as empty cells.
    values: ReadSignal<Vec<Vec<f64>>>,
    /// One label per row, down the left.
    rows: ReadSignal<Vec<String>>,
    /// One label per column, along the top.
    columns: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Sessions by weekday and hour". Used as the accessible name.
    label: String,
    /// Cell height in px.
    #[props(default = 36)]
    cell_height: u16,
    /// Print each value inside its cell.
    #[props(default)]
    show_values: bool,
    /// Formats values for the cells, tooltip, legend and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let rows_v = rows();
    let cols_v = columns();
    let l = heatmap::layout(&values());
    let mut active = use_signal(|| None::<(usize, usize)>);
    let (n_rows, n_cols) = (l.rows, l.cols);
    let current = active().filter(|&(r, c)| r < n_rows && c < n_cols);
    let name = |list: &[String], i: usize| list.get(i).cloned().unwrap_or_default();
    let value_at = |r: usize, c: usize| l.cells.get(r * n_cols + c).and_then(|cell| cell.value);
    let announcement = current.map(|(r, c)| {
        format!("{}, {}: {}", name(&rows_v, r), name(&cols_v, c), value_at(r, c).map_or_else(|| "no data".to_string(), fmt))
    });

    rsx! {
        figure { "data-scope": "heatmap-chart", "data-part": "root",
            div { "data-part": "frame", style: "--cols: {n_cols.max(1)}; --rows: {n_rows.max(1)}; --cell: {cell_height}px;",
                div { "data-part": "col-labels", "aria-hidden": "true",
                    for c in cols_v.iter() {
                        span { "data-part": "label", title: "{c}", "{c}" }
                    }
                }
                div { "data-part": "row-labels", "aria-hidden": "true",
                    for r in rows_v.iter() {
                        span { "data-part": "label", title: "{r}", "{r}" }
                    }
                }
                div {
                    "data-part": "cells",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to move between cells.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = heatmap::move_key(&e.key().to_string(), active(), n_rows, n_cols) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for cell in l.cells.iter() {
                        span {
                            key: "{cell.row}-{cell.col}",
                            "data-part": "cell",
                            "data-empty": cell.value.is_none().then_some(""),
                            "data-strong": (cell.level > 0.6).then_some(""),
                            "data-active": (current == Some((cell.row, cell.col))).then_some(""),
                            style: "--level: {cell.level:.3}; --i: {cell.row + cell.col};",
                            onpointerenter: {
                                let at = (cell.row, cell.col);
                                move |_| active.set(Some(at))
                            },
                            if show_values {
                                if let Some(v) = cell.value {
                                    {fmt(v)}
                                }
                            }
                        }
                    }
                    if let Some((r , c)) = current {
                        div {
                            "data-part": "tooltip",
                            "data-side": if r == 0 { "below" } else { "above" },
                            style: "{tip_style(r, c, n_rows, n_cols)}",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label", "{name(&rows_v, r)} · {name(&cols_v, c)}" }
                            div { "data-part": "tooltip-value", "{value_at(r, c).map_or_else(|| \"No data\".to_string(), fmt)}" }
                        }
                    }
                }
            }
            div { "data-part": "scale", "aria-hidden": "true",
                span { "data-part": "scale-value", {fmt(l.min)} }
                span { "data-part": "ramp" }
                span { "data-part": "scale-value", {fmt(l.max)} }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement.clone().unwrap_or_default()}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
                        for c in cols_v.iter() {
                            th { scope: "col", "{c}" }
                        }
                    }
                }
                tbody {
                    for (r , rname) in rows_v.iter().enumerate() {
                        tr {
                            th { scope: "row", "{rname}" }
                            for c in 0..n_cols {
                                td { "{value_at(r, c).map_or_else(|| \"–\".to_string(), fmt)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
