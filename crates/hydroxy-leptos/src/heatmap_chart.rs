//! `<HeatmapChart>`: a rows × columns grid whose cells shade by value. The scale is one
//! chart colour mixed into the surface, so it follows the theme. Geometry comes from
//! `hydroxy_charts::heatmap`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{format_value, heatmap, line, nav, raster};
use leptos::either::Either;
use leptos::prelude::*;

use crate::chart_frame::provide_export;
use crate::line_chart::plot_fractions;
use crate::plot_kit::pct;
use crate::raster_canvas::{Frame, RasterCanvas};

/// Steps in the canvas colour ramp; the DOM cells mix the same colours continuously.
const RAMP_STEPS: usize = 48;

/// A grid of values, one `Vec` per row, shaded from the lowest to the highest value.
/// Grids of more than a few thousand cells draw on a canvas with sparse labels, so
/// hundreds of rows and columns stay fast.
/// Hover or touch a cell to read it; focus the grid and use the arrow keys, Home and End to move,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "grid sizes are small and fractions clamped"
)]
#[component]
pub fn HeatmapChart(
    /// Values by row, then column. Missing or non-finite entries draw as empty cells.
    #[prop(into)]
    values: Signal<Vec<Vec<f64>>>,
    /// One label per row, down the left.
    #[prop(into)]
    rows: Signal<Vec<String>>,
    /// One label per column, along the top.
    #[prop(into)]
    columns: Signal<Vec<String>>,
    /// What the chart shows, such as "Sessions by weekday and hour". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Cell height in px.
    #[prop(default = 36)]
    cell_height: u16,
    /// Print each value inside its cell.
    #[prop(optional)]
    show_values: bool,
    /// Formats values for the cells, tooltip, legend and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a cell, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a cell.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| values.with(|v| heatmap::layout(v)));
    let big = Memo::new(move |_| layout.with(|l| l.cells.len() > raster::DOM_LIMIT));
    let active = RwSignal::new(None::<(usize, usize)>);
    let hover = move |ev: leptos::ev::PointerEvent| {
        if !big.get_untracked() {
            return;
        }
        let Some((x, y)) = plot_fractions(&ev) else { return };
        let (rows, cols) = layout.with(|l| (l.rows, l.cols));
        let at = |f: f64, n: usize| ((f.clamp(0.0, 1.0) * n as f64) as usize).min(n.saturating_sub(1));
        let next = (rows > 0 && cols > 0).then(|| (at(1.0 - y, rows), at(x, cols)));
        if active.get_untracked() != next {
            active.set(next);
        }
    };
    let ramp = Signal::derive(|| {
        (0..RAMP_STEPS)
            .map(|i| {
                let level = i as f64 / (RAMP_STEPS - 1) as f64;
                format!("color-mix(in oklab, var(--chart-1) {:.2}%, var(--surface-sunken))", 8.0 + level * 92.0)
            })
            .collect::<Vec<_>>()
    });
    let draw = Callback::new(move |f: Frame| layout.with(|l| raster::heat(l, &f.colours, f.width, f.height)));
    provide_export(move || {
        let l = layout.get_untracked();
        let head = std::iter::once("Row".to_string()).chain(columns.get_untracked()).collect::<Vec<_>>();
        let body = rows.get_untracked().into_iter().enumerate().map(|(r, name)| {
            std::iter::once(name)
                .chain(
                    (0..l.cols)
                        .map(|c| l.cells.get(r * l.cols + c).and_then(|cell| cell.value).map_or_else(String::new, fmt)),
                )
                .collect::<Vec<_>>()
        });
        std::iter::once(head).chain(body).collect()
    });
    // Sparse, absolutely placed labels for canvas grids.
    let sparse = move |list: Signal<Vec<String>>, n: usize, axis: &'static str| {
        list.with(|l| {
            line::label_indices(n.min(l.len()), 12)
                .into_iter()
                .map(|i| {
                    let at = pct((i as f64 + 0.5) / n.max(1) as f64);
                    let style = if axis == "x" { format!("left: {at};") } else { format!("top: {at};") };
                    let (t, title) = (l[i].clone(), l[i].clone());
                    view! { <span data-part="label" data-at="" style=style title=title>{t}</span> }
                })
                .collect_view()
        })
    };
    let name = move |list: Signal<Vec<String>>, i: usize| list.with(|l| l.get(i).cloned().unwrap_or_default());
    let datum = Signal::derive(move || {
        let (r, c) = active.get()?;
        let value = layout.with(|l| l.cells.get(r * l.cols + c).and_then(|cell| cell.value))?;
        Some(EventDatum { series: r, index: c, label: Some(name(columns, c)), value, node: None })
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let describe = move |(r, c): (usize, usize)| {
        let v = layout.with(|l| l.cells.get(r * l.cols + c).and_then(|cell| cell.value));
        format!("{}, {}: {}", name(rows, r), name(columns, c), v.map_or_else(|| "no data".to_string(), fmt))
    };

    view! {
        <figure data-scope="heatmap-chart" data-part="root">
            <div
                data-part="frame"
                data-raster=move || big.get().then_some("")
                style=move || {
                    let l = layout.get();
                    format!("--cols: {}; --rows: {}; --cell: {cell_height}px;", l.cols.max(1), l.rows.max(1))
                }
            >
                <div data-part="col-labels" aria-hidden="true">
                    {move || {
                        if big.get() {
                            return Either::Left(sparse(columns, layout.with(|l| l.cols), "x"));
                        }
                        Either::Right(columns.get().into_iter().map(|c| { let t = c.clone(); view! { <span data-part="label" title=t>{c}</span> } }).collect_view())
                    }}
                </div>
                <div data-part="row-labels" aria-hidden="true">
                    {move || {
                        if big.get() {
                            let widest = rows.with(|r| r.iter().max_by_key(|r| r.chars().count()).cloned().unwrap_or_default());
                            return Either::Left(view! {
                                <span data-part="sizer">{widest}</span>
                                {sparse(rows, layout.with(|l| l.rows), "y")}
                            });
                        }
                        Either::Right(rows.get().into_iter().map(|r| { let t = r.clone(); view! { <span data-part="label" title=t>{r}</span> } }).collect_view())
                    }}
                </div>
                <div
                    data-part="cells"
                    tabindex="0"
                    role="group"
                    aria-label=nav::hinted(&label, nav::HINT_CELLS)
                    on:pointermove=hover
                    on:pointerdown=hover
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        let (r, c) = layout.with(|l| (l.rows, l.cols));
                        if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), r, c) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    <Show when=move || big.get()>
                        <RasterCanvas colours=ramp draw />
                        {move || {
                            let (r, c) = active.get()?;
                            let (rows, cols) = layout.with(|l| (l.rows.max(1) as f64, l.cols.max(1) as f64));
                            Some(view! {
                                <div
                                    data-part="highlight"
                                    style=format!(
                                        "left: {:.3}%; top: {:.3}%; width: {:.3}%; height: {:.3}%;",
                                        c as f64 / cols * 100.0,
                                        r as f64 / rows * 100.0,
                                        100.0 / cols,
                                        100.0 / rows,
                                    )
                                ></div>
                            })
                        }}
                    </Show>
                    {move || {
                        if big.get() {
                            return Vec::new();
                        }
                        layout
                            .get()
                            .cells
                            .into_iter()
                            .map(|cell| {
                                let at = (cell.row, cell.col);
                                view! {
                                    <span
                                        data-part="cell"
                                        data-empty=cell.value.is_none().then_some("")
                                        data-strong=(cell.level > 0.6).then_some("")
                                        data-active=move || (active.get() == Some(at)).then_some("")
                                        style=format!("--level: {:.3}; --i: {};", cell.level, cell.row + cell.col)
                                        on:pointerenter=move |_| active.set(Some(at))
                                        on:pointerdown=move |_| active.set(Some(at))
                                    >
                                        {show_values.then(|| cell.value.map(fmt))}
                                    </span>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                    {move || {
                        let (r, c) = active.get()?;
                        let l = layout.get();
                        let value = l.cells.get(r * l.cols + c).and_then(|cell| cell.value);
                        Some(
                            view! {
                                <div
                                    data-part="tooltip"
                                    data-side={if r == 0 { "below" } else { "above" }}
                                    style=format!(
                                        "left: {:.3}%; top: {:.3}%;",
                                        (c as f64 + 0.5) / l.cols as f64 * 100.0,
                                        (r as f64 + if r == 0 { 1.0 } else { 0.0 }) / l.rows as f64 * 100.0,
                                    )
                                    aria-hidden="true"
                                >
                                    <div data-part="tooltip-label">{format!("{} · {}", name(rows, r), name(columns, c))}</div>
                                    <div data-part="tooltip-value">{value.map_or_else(|| "No data".to_string(), fmt)}</div>
                                </div>
                            },
                        )
                    }}
                </div>
            </div>
            <div data-part="scale" aria-hidden="true">
                <span data-part="scale-value">{move || fmt(layout.with(|l| l.min))}</span>
                <span data-part="ramp"></span>
                <span data-part="scale-value">{move || fmt(layout.with(|l| l.max))}</span>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>
                    {label}
                    {move || {
                        let (shown, total) = layout.with(|l| (raster::table_rows(l.rows, l.cols), l.rows));
                        (shown < total).then(|| format!(" (first {shown} of {total} rows)"))
                    }}
                </caption>
                <thead>
                    <tr>
                        <th scope="col">"Row"</th>
                        {move || columns.get().into_iter().map(|c| view! { <th scope="col">{c}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        let shown = raster::table_rows(l.rows, l.cols);
                        rows.get()
                            .into_iter()
                            .take(shown)
                            .enumerate()
                            .map(|(r, name)| {
                                let cells = (0..l.cols)
                                    .map(|c| {
                                        let v = l.cells.get(r * l.cols + c).and_then(|cell| cell.value);
                                        view! { <td>{v.map_or_else(|| "–".to_string(), fmt)}</td> }
                                    })
                                    .collect_view();
                                view! { <tr><th scope="row">{name}</th>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
