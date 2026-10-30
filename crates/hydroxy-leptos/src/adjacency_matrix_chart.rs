//! `<AdjacencyMatrixChart>`: a nodes × nodes grid, each cell shaded by the weight of
//! the link between its row and column. Ordering comes from `hydroxy_charts::matrix`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::{self, HINT_CELLS, hinted};
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, matrix};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
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

/// An adjacency matrix. Hover a cell to read the link and light its row and column;
/// focus the grid and use the arrow keys, Home and End to move, Escape to clear.
#[component]
pub fn AdjacencyMatrixChart(
    /// The nodes. Ids must be unique; they are also the labels.
    #[prop(into)]
    nodes: Signal<Vec<Node>>,
    /// The links, by node id. Repeated links add up; unknown ids are ignored.
    #[prop(into)]
    links: Signal<Vec<Link>>,
    /// What the chart shows, such as "Messages between teams". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Row and column order: by name (default), degree or cluster.
    #[prop(optional)]
    order: MatrixOrder,
    /// Links run one way. When false each link fills both mirrored cells.
    #[prop(optional)]
    directed: bool,
    /// Cell size in px.
    #[prop(default = 28)]
    cell_size: u16,
    /// Formats values for the tooltip, readout and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| nodes.with(|n| links.with(|l| matrix::layout(n, l, order, directed))));
    let active = RwSignal::new(None::<(usize, usize)>);
    let size = move || layout.with(|l| l.order.len());
    let id = move |display: usize| {
        layout
            .with(|l| l.order.get(display).and_then(|&i| nodes.with(|n| n.get(i).map(|n| n.id.clone()))))
            .unwrap_or_default()
    };
    let arrow = if directed { "→" } else { "–" };
    let value_at = move |r: usize, c: usize| layout.with(|l| l.cells.get(r * l.order.len() + c).and_then(|c| c.value));
    let describe = move |(r, c): (usize, usize)| {
        format!("{} {arrow} {}: {}", id(r), id(c), value_at(r, c).map_or_else(|| "no link".to_string(), fmt))
    };
    // Series is the row and index the column, both in display order; an empty cell reports NaN.
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let (r, c) = active.get()?;
            let value = value_at(r, c).unwrap_or(f64::NAN);
            Some(EventDatum {
                series: r,
                index: c,
                label: Some(format!("{} {arrow} {}", id(r), id(c))),
                value,
                node: None,
            })
        })
    });
    watch_hover(datum, on_hover);
    let lit =
        move |i: usize, row: bool| active.get().is_some_and(|(r, c)| if row { r == i } else { c == i }).then_some("");
    let caption = label.clone();

    view! {
        <figure data-scope="adjacency-matrix-chart" data-part="root">
            <div data-part="plot" style=move || format!("--n: {}; --cell: {cell_size}px;", size().max(1))>
                <div data-part="col-labels" aria-hidden="true">
                    {move || (0..size()).map(|c| view! { <span data-part="col-label" data-active=move || lit(c, false)>{id(c)}</span> }).collect_view()}
                </div>
                <div data-part="row-labels" aria-hidden="true">
                    {move || (0..size()).map(|r| view! { <span data-part="row-label" data-active=move || lit(r, true)>{id(r)}</span> }).collect_view()}
                </div>
                <div
                    data-part="cells"
                    tabindex="0"
                    role="group"
                    aria-label=hinted(&label, HINT_CELLS)
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            select_active(datum, on_select);
                        }
                        let n = size();
                        if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), n, n) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let cells = layout.get().cells;
                        let n = cells.iter().map(|c| c.row + c.col + 1).max().unwrap_or(1);
                        cells
                            .into_iter()
                            .map(|cell| {
                                let at = (cell.row, cell.col);
                                view! {
                                    <span
                                        data-part="cell"
                                        data-empty=cell.value.is_none().then_some("")
                                        data-active=move || (active.get() == Some(at)).then_some("")
                                        data-cross=move || active.get().is_some_and(|(r, c)| r == at.0 || c == at.1).then_some("")
                                        style=format!("--level: {:.3}; --i: {}; --ox-n: {n};", cell.level, cell.row + cell.col)
                                        on:pointerenter=move |_| active.set(Some(at))
                                        on:pointerdown=move |_| active.set(Some(at))
                                        on:click=move |_| {
                                            active.set(Some(at));
                                            select_active(datum, on_select);
                                        }
                                    ></span>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let (r, c) = active.get()?;
                        let (style, left, below) = tip_style(r, c, size());
                        // The anchor only picks the side and vertical placement; `style` positions it.
                        let tip = Tip {
                            x: if left { 1.0 } else { 0.0 },
                            y: if below { 1.0 } else { 0.0 },
                            title: format!("{} {arrow} {}", id(r), id(c)),
                            rows: vec![TipRow::new(None, "Value", value_at(r, c).map_or_else(|| "No link".to_string(), fmt))],
                        };
                        Some(view! { <ChartTip tip swatch=colour(0) style=style /> })
                    }}
                </div>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Source"</th>
                        <th scope="col">"Target"</th>
                        <th scope="col">"Value"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let known: Vec<String> = nodes.with(|n| n.iter().map(|n| n.id.clone()).collect());
                        links
                            .get()
                            .into_iter()
                            .filter(|l| known.contains(&l.source) && known.contains(&l.target))
                            .map(|l| view! { <tr><th scope="row">{l.source}</th><td>{l.target}</td><td>{fmt(l.value)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
