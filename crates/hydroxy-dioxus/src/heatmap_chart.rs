//! `HeatmapChart`: a rows × columns grid whose cells shade by value. The scale is one
//! chart colour mixed into the surface, so it follows the theme. Geometry comes from
//! `hydroxy_charts::heatmap`.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, heatmap, line, nav, raster};

use crate::chart_frame::use_export;
use crate::line_chart::plot_fractions;
use crate::raster_canvas::{Frame, RasterCanvas};

/// Steps in the canvas colour ramp; the DOM cells mix the same colours continuously.
const RAMP_STEPS: usize = 48;

#[allow(clippy::cast_precision_loss, reason = "grid sizes are tiny")]
fn tip_style(r: usize, c: usize, rows: usize, cols: usize) -> String {
    let below = if r == 0 { 1.0 } else { 0.0 };
    format!("left: {:.3}%; top: {:.3}%;", (c as f64 + 0.5) / cols as f64 * 100.0, (r as f64 + below) / rows as f64 * 100.0)
}

/// A grid of values, one `Vec` per row, shaded from the lowest to the highest value.
/// Grids of more than a few thousand cells draw on a canvas with sparse labels, so
/// hundreds of rows and columns stay fast.
/// Hover or touch a cell to read it; focus the grid and use the arrow keys, Home and End to move,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "grid sizes are small and fractions clamped")]
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
    // A memo, so a hover never re-lays out the grid.
    let layout = use_memo(move || heatmap::layout(&values.read()));
    let mut active = use_signal(|| None::<(usize, usize)>);
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let draw = use_callback(move |f: Frame| raster::heat(&layout.read(), &f.colours, f.width, f.height));
    use_export(move |()| {
        let l = layout.read();
        let head = std::iter::once("Row".to_string()).chain(columns.read().iter().cloned()).collect::<Vec<_>>();
        let rows = rows.read();
        let body = rows.iter().enumerate().map(|(r, name)| {
            std::iter::once(name.clone())
                .chain((0..l.cols).map(|c| l.cells.get(r * l.cols + c).and_then(|cell| cell.value).map_or_else(String::new, fmt)))
                .collect::<Vec<_>>()
        });
        std::iter::once(head).chain(body).collect()
    });
    let rows_v = rows.read();
    let cols_v = columns.read();
    let l = layout.read();
    let (n_rows, n_cols) = (l.rows, l.cols);
    let big = l.cells.len() > raster::DOM_LIMIT;
    let ramp: Vec<String> = (0..RAMP_STEPS)
        .map(|i| {
            let level = i as f64 / (RAMP_STEPS - 1) as f64;
            format!("color-mix(in oklab, var(--chart-1) {:.2}%, var(--surface-sunken))", 8.0 + level * 92.0)
        })
        .collect();
    // Sparse, absolutely placed labels for canvas grids.
    let sparse = |list: &[String], n: usize, axis: &str| -> Vec<(String, String)> {
        line::label_indices(n.min(list.len()), 12)
            .into_iter()
            .map(|i| {
                let at = format!("{:.3}%", (i as f64 + 0.5) / n.max(1) as f64 * 100.0);
                (if axis == "x" { format!("left: {at};") } else { format!("top: {at};") }, list[i].clone())
            })
            .collect()
    };
    let mut hover = move |e: Event<PointerData>| {
        let Some((x, y)) = plot_fractions(e.data().element_coordinates(), size()) else { return };
        let at = |f: f64, n: usize| ((f.clamp(0.0, 1.0) * n as f64) as usize).min(n.saturating_sub(1));
        let next = (n_rows > 0 && n_cols > 0).then(|| (at(1.0 - y, n_rows), at(x, n_cols)));
        if active() != next {
            active.set(next);
        }
    };
    let shown = raster::table_rows(n_rows, n_cols);
    let current = active().filter(|&(r, c)| r < n_rows && c < n_cols);
    let name = |list: &[String], i: usize| list.get(i).cloned().unwrap_or_default();
    let value_at = |r: usize, c: usize| l.cells.get(r * n_cols + c).and_then(|cell| cell.value);
    let announcement = current.map(|(r, c)| {
        format!("{}, {}: {}", name(&rows_v, r), name(&cols_v, c), value_at(r, c).map_or_else(|| "no data".to_string(), fmt))
    });

    rsx! {
        figure { "data-scope": "heatmap-chart", "data-part": "root",
            div {
                "data-part": "frame",
                "data-raster": big.then_some(""),
                style: "--cols: {n_cols.max(1)}; --rows: {n_rows.max(1)}; --cell: {cell_height}px;",
                div { "data-part": "col-labels", "aria-hidden": "true",
                    if big {
                        for (style , t) in sparse(&cols_v, n_cols, "x") {
                            span { "data-part": "label", "data-at": "", style, title: "{t}", "{t}" }
                        }
                    } else {
                        for c in cols_v.iter() {
                            span { "data-part": "label", title: "{c}", "{c}" }
                        }
                    }
                }
                div { "data-part": "row-labels", "aria-hidden": "true",
                    if big {
                        span { "data-part": "sizer", {rows_v.iter().max_by_key(|r| r.chars().count()).cloned().unwrap_or_default()} }
                        for (style , t) in sparse(&rows_v, n_rows, "y") {
                            span { "data-part": "label", "data-at": "", style, title: "{t}", "{t}" }
                        }
                    } else {
                        for r in rows_v.iter() {
                            span { "data-part": "label", title: "{r}", "{r}" }
                        }
                    }
                }
                div {
                    "data-part": "cells",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to move between cells.",
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            size.set((s.width, s.height));
                        }
                    },
                    onpointermove: move |e| {
                        if big {
                            hover(e);
                        }
                    },
                    onpointerdown: move |e| {
                        if big {
                            hover(e);
                        }
                    },
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::grid_key(&e.key().to_string(), active(), n_rows, n_cols) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    if big {
                        RasterCanvas { colours: ramp, draw }
                        if let Some((r , c)) = current {
                            div {
                                "data-part": "highlight",
                                style: "left: {c as f64 / n_cols.max(1) as f64 * 100.0:.3}%; top: {r as f64 / n_rows.max(1) as f64 * 100.0:.3}%; width: {100.0 / n_cols.max(1) as f64:.3}%; height: {100.0 / n_rows.max(1) as f64:.3}%;",
                            }
                        }
                    }
                    for cell in l.cells.iter().filter(|_| !big) {
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
                            onpointerdown: {
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
                caption {
                    "{label}"
                    if shown < n_rows {
                        " (first {shown} of {n_rows} rows)"
                    }
                }
                thead {
                    tr {
                        th { scope: "col", "Row" }
                        for c in cols_v.iter() {
                            th { scope: "col", "{c}" }
                        }
                    }
                }
                tbody {
                    for (r , rname) in rows_v.iter().take(shown).enumerate() {
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
