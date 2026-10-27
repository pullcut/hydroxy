//! `AdjacencyMatrixChart`: a nodes × nodes grid, each cell shaded by the weight of the
//! link between its row and column. Ordering comes from `oxide_charts::matrix`.

use dioxus::prelude::*;
use oxide_charts::{format_value, matrix, nav};

pub use oxide_charts::matrix::MatrixOrder;
pub use oxide_charts::network::{Link, Node};

#[allow(clippy::cast_precision_loss, reason = "grid sizes are tiny")]
fn tip_style(row: usize, col: usize, size: usize) -> (String, bool, bool) {
    let size = size.max(1) as f64;
    let below = (row as f64) / size < 0.25;
    let left = (col as f64 + 0.5) / size * 100.0;
    let top = (row as f64 + f64::from(u8::from(below))) / size * 100.0;
    (format!("left: {left:.2}%; top: {top:.2}%;"), left > 50.0, below)
}

/// An adjacency matrix. Hover a cell to read the link and light its row and column;
/// focus the grid and use the arrow keys, Home and End to move, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn AdjacencyMatrixChart(
    /// The nodes. Ids must be unique; they are also the labels.
    nodes: ReadSignal<Vec<Node>>,
    /// The links, by node id. Repeated links add up; unknown ids are ignored.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Messages between teams". Used as the accessible name.
    label: String,
    /// Row and column order: by name (default), degree or cluster.
    #[props(default)]
    order: MatrixOrder,
    /// Links run one way. When false each link fills both mirrored cells.
    #[props(default)]
    directed: bool,
    /// Cell size in px.
    #[props(default = 28)]
    cell_size: u16,
    /// Formats values for the tooltip, readout and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let nodes_v = nodes();
    let links_v = links();
    let lay = matrix::layout(&nodes_v, &links_v, order, directed);
    let mut active = use_signal(|| None::<(usize, usize)>);
    let size = lay.order.len();
    let current = active().filter(|&(r, c)| r < size && c < size);
    let id = |display: usize| lay.order.get(display).map(|&i| nodes_v[i].id.clone()).unwrap_or_default();
    let arrow = if directed { "→" } else { "–" };
    let value_at = |r: usize, c: usize| lay.cells.get(r * size + c).and_then(|cell| cell.value);
    let shown = |r: usize, c: usize| value_at(r, c).map_or_else(|| "no link".to_string(), fmt);
    let announcement = current.map(|(r, c)| format!("{} {arrow} {}: {}", id(r), id(c), shown(r, c))).unwrap_or_default();
    let lit = |i: usize, row: bool| current.is_some_and(|(r, c)| if row { r == i } else { c == i }).then_some("");
    let known = |s: &str| nodes_v.iter().any(|n| n.id == s);

    rsx! {
        figure { "data-scope": "adjacency-matrix-chart", "data-part": "root",
            div { "data-part": "plot", style: "--n: {size.max(1)}; --cell: {cell_size}px;",
                div { "data-part": "col-labels", "aria-hidden": "true",
                    for c in 0..size {
                        span { "data-part": "col-label", "data-active": lit(c, false), "{id(c)}" }
                    }
                }
                div { "data-part": "row-labels", "aria-hidden": "true",
                    for r in 0..size {
                        span { "data-part": "row-label", "data-active": lit(r, true), "{id(r)}" }
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
                        if let Some(next) = nav::grid_key(&e.key().to_string(), active(), size, size) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for cell in lay.cells.iter() {
                        span {
                            key: "{cell.row}-{cell.col}",
                            "data-part": "cell",
                            "data-empty": cell.value.is_none().then_some(""),
                            "data-active": (current == Some((cell.row, cell.col))).then_some(""),
                            "data-cross": current.is_some_and(|(r, c)| r == cell.row || c == cell.col).then_some(""),
                            style: "--level: {cell.level:.3}; --i: {cell.row + cell.col};",
                            onpointerenter: {
                                let at = (cell.row, cell.col);
                                move |_| active.set(Some(at))
                            },
                            onpointerdown: {
                                let at = (cell.row, cell.col);
                                move |_| active.set(Some(at))
                            },
                        }
                    }
                    if let Some((r , c)) = current {
                        div {
                            "data-part": "tooltip",
                            "data-side": if tip_style(r, c, size).1 { "left" } else { "right" },
                            "data-vertical": if tip_style(r, c, size).2 { "below" } else { "above" },
                            style: "{tip_style(r, c, size).0}",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label",
                                span { "data-part": "swatch", style: "background: var(--chart-1);" }
                                "{id(r)} {arrow} {id(c)}"
                            }
                            ul {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "Value" }
                                    span { "data-part": "tooltip-value", "{value_at(r, c).map_or_else(|| \"No link\".to_string(), fmt)}" }
                                }
                            }
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Source" }
                        th { scope: "col", "Target" }
                        th { scope: "col", "Value" }
                    }
                }
                tbody {
                    for l in links_v.iter().filter(|l| known(&l.source) && known(&l.target)) {
                        tr {
                            th { scope: "row", "{l.source}" }
                            td { "{l.target}" }
                            td { {fmt(l.value)} }
                        }
                    }
                }
            }
        }
    }
}
