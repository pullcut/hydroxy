//! `AdjacencyMatrixChart`: a nodes × nodes grid, each cell shaded by the weight of the
//! link between its row and column. Ordering comes from `hydroxy_charts::matrix`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, matrix, nav};

use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::matrix::MatrixOrder;
pub use hydroxy_charts::network::{Link, Node};

#[allow(clippy::cast_precision_loss, reason = "grid sizes are tiny")]
fn tip_style(row: usize, col: usize, size: usize) -> (String, bool, bool) {
    let size = size.max(1) as f64;
    let below = (row as f64) / size < 0.25;
    let left = (col as f64 + 0.5) / size * 100.0;
    let top = (row as f64 + f64::from(u8::from(below))) / size * 100.0;
    (format!("left: {left:.2}%; top: {top:.2}%;"), left > 50.0, below)
}

/// The tooltip content. Position and side come from [`tip_style`]; `y` only carries the
/// `data-vertical` choice (`below` when above 0.5).
fn cell_tip(row: usize, col: usize, size: usize, title: String, value: String) -> Tip {
    let (_, _, below) = tip_style(row, col, size);
    Tip { x: 0.0, y: if below { 1.0 } else { 0.0 }, title, rows: vec![TipRow::new(None, "Value", value)] }
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
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
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
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|(r, c)| EventDatum {
            series: r,
            index: c,
            label: Some(format!("{} {arrow} {}", id(r), id(c))),
            value: value_at(r, c).unwrap_or(f64::NAN),
            node: None,
        })
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let shown = |r: usize, c: usize| value_at(r, c).map_or_else(|| "no link".to_string(), fmt);
    let announcement =
        current.map(|(r, c)| format!("{} {arrow} {}: {}", id(r), id(c), shown(r, c))).unwrap_or_default();
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
                    "aria-label": "{nav::hinted(&label, nav::HINT_CELLS)}",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
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
                            onclick: {
                                let at = (cell.row, cell.col);
                                move |_| {
                                    active.set(Some(at));
                                    crate::callbacks::select_active(Some(datum), on_select);
                                }
                            },
                        }
                    }
                    if let Some((r , c)) = current {
                        ChartTip {
                            tip: cell_tip(r, c, size, format!("{} {arrow} {}", id(r), id(c)), value_at(r, c).map_or_else(|| "No link".to_string(), fmt)),
                            swatch: colour(0),
                            style: tip_style(r, c, size).0,
                            side: if tip_style(r, c, size).1 { "left" } else { "right" },
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
