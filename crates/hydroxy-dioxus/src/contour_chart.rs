//! `ContourChart` and `HeatField`: a scalar field on a grid, as contour lines
//! (marching squares) and/or a filled heat field. Geometry comes from `hydroxy_charts::field`.

use dioxus::prelude::*;
use hydroxy_charts::{field, format_value, nav};

/// Contour lines over a grid of values, optionally on a filled heat field. Values are
/// row-major, `columns` per row; NaN leaves a gap. Hover or touch to read a cell; focus
/// the field and use the arrow keys to move between cells, Home and End to jump along
/// the row, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "grid indices are small")]
#[component]
pub fn ContourChart(
    /// Grid values, row-major.
    values: ReadSignal<Vec<f64>>,
    /// Values per row.
    columns: usize,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// About how many contour levels to draw.
    #[props(default = 6)]
    levels: usize,
    /// Shade each cell by its value underneath the lines.
    #[props(default)]
    filled: bool,
    /// Formats values on the legend, tooltips and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let show = move |v: Option<f64>| v.filter(|v| v.is_finite()).map_or_else(|| "no data".to_string(), fmt);
    let vals = values();
    let cols = columns.max(1);
    let rows = vals.len().div_ceil(cols);
    let l = field::layout(&vals, columns, levels);
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let span = (l.range.1 - l.range.0).max(f64::EPSILON);
    let mut active = use_signal(|| None::<(usize, usize)>);
    let mut size = use_signal(|| (1.0_f64, 1.0_f64));
    let current = active().filter(|&(r, c)| r < rows && c < cols);
    let value_at = |(r, c): (usize, usize)| vals.get(r * cols + c).copied();
    let pick = move |e: Event<PointerData>| {
        let (w, h) = size();
        let p = e.data().element_coordinates();
        let next = field::cell_at(p.x / w.max(1.0), p.y / h.max(1.0), rows, cols);
        if active() != next {
            active.set(next);
        }
    };
    let announcement = current.map(|(r, c)| format!("Row {}, column {}: {}", r + 1, c + 1, show(value_at((r, c)))));
    let tip = current.map(|(r, c)| ((r, c), ((c as f64 + 0.5) / cols as f64, (r as f64 + 0.5) / rows.max(1) as f64)));

    rsx! {
        figure { "data-scope": "contour-chart", "data-part": "root", "data-filled": filled.then_some(""),
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to move between cells.",
                onresize: move |e| {
                    if let Ok(s) = e.data().get_border_box_size() {
                        size.set((s.width, s.height));
                    }
                },
                onpointermove: pick,
                onpointerdown: pick,
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::grid_key(&e.key().to_string(), active(), rows, cols) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { "data-part": "marks", view_box: "{view_box}", "aria-hidden": "true",
                    if filled {
                        for (i , t) in l.heat.iter().enumerate() {
                            if let Some(t) = t {
                                rect {
                                    key: "{i}",
                                    "data-part": "cell",
                                    x: "{(i % cols) as f64 * l.cell}",
                                    y: "{(i / cols) as f64 * l.cell}",
                                    width: "{l.cell}",
                                    height: "{l.cell}",
                                    style: "--t: {t:.3};",
                                }
                            }
                        }
                    }
                    for c in l.contours.iter() {
                        path {
                            "data-part": "contour",
                            d: "{c.path}",
                            "pathLength": "1",
                            style: "--t: {(c.level - l.range.0) / span:.3};",
                        }
                    }
                    if let Some((r, c)) = current {
                        rect {
                            "data-part": "active-cell",
                            x: "{c as f64 * l.cell}",
                            y: "{r as f64 * l.cell}",
                            width: "{l.cell}",
                            height: "{l.cell}",
                        }
                    }
                }
                if let Some(((r, c), (x, y))) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if x > 0.5 { "left" } else { "right" },
                        style: "left: {x * 100.0:.3}%; top: {y * 100.0:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label", "Row {r + 1}, column {c + 1}" }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {show(value_at((r, c)))} }
                            }
                        }
                    }
                }
            }
            div { "data-part": "scale", "aria-hidden": "true",
                span { {fmt(l.range.0)} }
                span { "data-part": "ramp" }
                span { {fmt(l.range.1)} }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Row" }
                        for c in 1..=cols {
                            th { scope: "col", "Column {c}" }
                        }
                    }
                }
                tbody {
                    for (r , row) in vals.chunks(cols).enumerate() {
                        tr {
                            th { scope: "row", "Row {r + 1}" }
                            for x in row.iter() {
                                td { if x.is_finite() { {fmt(*x)} } else { "–" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A filled heat field with no contour lines: a smooth-ish picture of a grid of values.
/// Reads and navigates like [`ContourChart`].
#[component]
pub fn HeatField(
    /// Grid values, row-major.
    values: ReadSignal<Vec<f64>>,
    /// Values per row.
    columns: usize,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Formats values on the legend, tooltips and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    rsx! { ContourChart { values, columns, label, levels: 0, filled: true, format } }
}
