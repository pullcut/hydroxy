//! `WaffleChart`: a grid of unit cells (10×10 by default), each given to a category
//! in proportion to its value. Counts come from `oxide_charts::waffle`. More than six
//! categories fold into "Other".

use dioxus::prelude::*;
use oxide_charts::{fold_other, format_value, nav, waffle};

pub use oxide_charts::Datum;

fn slot_colour(i: usize, folded_other: bool, len: usize) -> String {
    if folded_other && i + 1 == len { "var(--chart-muted)".into() } else { format!("var(--chart-{})", i % 6 + 1) }
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

#[allow(clippy::cast_precision_loss, reason = "grid sizes are tiny")]
fn frac(i: usize, n: usize) -> f64 {
    (i as f64 + 0.5) / n.max(1) as f64
}

/// A waffle grid. Hover, touch or focus a cell to read its category; focus the grid
/// and use the arrow keys to move cell by cell, Home and End to jump along a row,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn WaffleChart(
    /// The categories, in fill order.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Budget by team". Used as the accessible name.
    label: String,
    /// The whole the grid stands for. Default: the sum of the values. A larger total
    /// leaves the remaining cells empty.
    total: Option<f64>,
    /// Cells per row.
    #[props(default = 10)]
    columns: usize,
    /// Rows of cells.
    #[props(default = 10)]
    rows: usize,
    /// Formats values for the tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (columns, rows) = (columns.max(1), rows.max(1));
    let cells_n = columns * rows;
    let raw = data();
    let folded = raw.len() > 6;
    let parts = fold_other(&raw, 6);
    let n = parts.len();
    let counts = waffle::counts(&parts.iter().map(|d| d.value).collect::<Vec<_>>(), total, cells_n);
    let grid = waffle::cells(&counts, cells_n);
    let mut active = use_signal(|| None::<(usize, usize)>);
    let current = active().filter(|&(r, c)| r < rows && c < columns);
    let category = current.and_then(|(r, c)| grid[r * columns + c]);
    let colour = |i: usize| slot_colour(i, folded, n);
    #[allow(clippy::cast_precision_loss, reason = "cell counts are tiny")]
    let share = |count: usize| format!("{}%", format_value(count as f64 / cells_n as f64 * 100.0));
    let announcement = match (current, category) {
        (Some(_), Some(k)) => format!("{}: {}, {} of {cells_n} cells", parts[k].label, fmt(parts[k].value), counts[k]),
        (Some((row, col)), None) => format!("Row {}, column {}: empty", row + 1, col + 1),
        _ => String::new(),
    };
    let first_cells: Vec<Option<(usize, usize)>> =
        (0..n).map(|k| grid.iter().position(|&x| x == Some(k)).map(|p| (p / columns, p % columns))).collect();

    rsx! {
        figure { "data-scope": "waffle-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to move between cells.",
                style: "--ox-cols: {columns}; --ox-rows: {rows};",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::grid_key(&e.key().to_string(), active(), rows, columns) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                for (i , k) in grid.iter().copied().enumerate() {
                    span {
                        key: "{i}",
                        "data-part": "cell",
                        "data-empty": k.is_none().then_some(""),
                        "data-active": (k.is_some() && category == k).then_some(""),
                        "data-focus": (current == Some((i / columns, i % columns))).then_some(""),
                        style: "--i: {i.min(60)}; background: {k.map_or_else(String::new, colour)};",
                        onpointerenter: move |_| active.set(Some((i / columns, i % columns))),
                        onpointerdown: move |_| active.set(Some((i / columns, i % columns))),
                    }
                }
                if let Some((row , col)) = current {
                    div {
                        "data-part": "tooltip",
                        "data-side": if frac(col, columns) > 0.5 { "left" } else { "right" },
                        style: "left: {pct(frac(col, columns))}; top: {pct(frac(row, rows))};",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {category.map_or_else(|| \"var(--chart-grid)\".into(), colour)};" }
                            {category.map_or_else(|| "Empty".to_string(), |k| parts[k].label.clone())}
                        }
                        ul {
                            if let Some(k) = category {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "Value" }
                                    span { "data-part": "tooltip-value", {fmt(parts[k].value)} }
                                }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Cells" }
                                span { "data-part": "tooltip-value",
                                    {category.map_or_else(|| "—".to_string(), |k| format!("{} ({})", counts[k], share(counts[k])))}
                                }
                            }
                        }
                    }
                }
            }
            if n > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , d) in parts.iter().enumerate() {
                        li {
                            "data-part": "legend-item",
                            "data-active": (category == Some(i)).then_some(""),
                            onpointerenter: {
                                let first = first_cells[i];
                                move |_| active.set(first)
                            },
                            onpointerleave: move |_| active.set(None),
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{d.label}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Category" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Cells" }
                    }
                }
                tbody {
                    for (d , count) in parts.iter().zip(counts.iter()) {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                            td { "{count}" }
                        }
                    }
                }
            }
        }
    }
}
