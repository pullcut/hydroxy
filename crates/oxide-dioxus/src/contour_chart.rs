//! `ContourChart` and `HeatField`: a scalar field on a grid, as contour lines
//! (marching squares) and/or a filled heat field. Geometry comes from `oxide_charts::field`.

use dioxus::prelude::*;
use oxide_charts::{field, format_value};

#[allow(clippy::cast_precision_loss, reason = "grid indices are small")]
/// Contour lines over a grid of values, optionally on a filled heat field. Values are
/// row-major, `columns` per row; NaN leaves a gap.
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
    /// Formats values on the legend and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let vals = values();
    let cols = columns.max(1);
    let l = field::layout(&vals, columns, levels);
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let span = (l.range.1 - l.range.0).max(f64::EPSILON);

    rsx! {
        figure { "data-scope": "contour-chart", "data-part": "root", "data-filled": filled.then_some(""),
            svg { "data-part": "marks", view_box: "{view_box}", role: "img", "aria-label": "{label}",
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
                    path { "data-part": "contour", d: "{c.path}", style: "--t: {(c.level - l.range.0) / span:.3};" }
                }
            }
            div { "data-part": "scale", "aria-hidden": "true",
                span { {fmt(l.range.0)} }
                span { "data-part": "ramp" }
                span { {fmt(l.range.1)} }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
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
#[component]
pub fn HeatField(
    /// Grid values, row-major.
    values: ReadSignal<Vec<f64>>,
    /// Values per row.
    columns: usize,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Formats values on the legend and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    rsx! { ContourChart { values, columns, label, levels: 0, filled: true, format } }
}
