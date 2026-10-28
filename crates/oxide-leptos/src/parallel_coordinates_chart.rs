//! `<ParallelCoordinatesChart>`: one vertical axis per dimension and one line per row
//! across them. Drag on an axis to brush a range; rows outside any brush dim. Geometry
//! comes from `oxide_charts::parallel`; thousands of rows draw on a canvas.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_charts::parallel::{self, Range};
use oxide_charts::plot::{self, Tip, TipRow};
use oxide_charts::{format_value, raster};

pub use oxide_charts::parallel::Dimension;

use crate::chart_frame::provide_export;
use crate::raster_canvas::{Frame, RasterCanvas};

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// How close to an axis, in px, a press starts a brush rather than reading a line.
const GRAB_PX: f64 = 14.0;

/// The pointer over `ev`'s target as plot fractions (y up), with the target's width in px.
fn pointer(ev: &leptos::ev::PointerEvent) -> Option<(f64, f64, f64)> {
    let el = ev.current_target()?.dyn_into::<web_sys::HtmlElement>().ok()?;
    let (w, h) = (f64::from(el.offset_width().max(1)), f64::from(el.offset_height().max(1)));
    Some((f64::from(ev.offset_x()) / w, 1.0 - f64::from(ev.offset_y()) / h, w))
}

/// One line per row across an axis per dimension, each axis on its own scale. Drag up or
/// down an axis to keep only rows in that range; brushes on several axes combine, and a
/// click on an axis clears its brush. Hover a line, or focus the plot and use Left/Right,
/// to read a row; Escape clears.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ParallelCoordinatesChart(
    /// The axes, left to right, each with one value per row.
    #[prop(into)]
    dimensions: Signal<Vec<Dimension>>,
    /// Each row's category, which picks its colour. Up to six read well; leave empty for one colour.
    #[prop(optional, into)]
    groups: Signal<Vec<String>>,
    /// Each row's name, for the tooltip and table. Default "Row 1", "Row 2"…
    #[prop(optional, into)]
    names: Signal<Vec<String>>,
    /// What the chart shows, such as "Cars by economy, power and weight". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis names and legend.
    #[prop(default = 320)]
    height: u16,
    /// Formats values for the ticks, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| dimensions.with(|d| parallel::layout(d)));
    let grouped = Memo::new(move |_| groups.with(|g| parallel::groups(g)));
    let rows = Memo::new(move |_| layout.with(|l| l.ys.len()));
    let big = Memo::new(move |_| rows.get() > raster::DOM_LIMIT);
    let brushes = RwSignal::new(Vec::<Option<Range>>::new());
    let keep = Memo::new(move |_| brushes.with(|b| layout.with(|l| parallel::selected(l, b))));
    let shown = Memo::new(move |_| keep.with(|k| k.iter().filter(|k| **k).count()));
    let filtered = Memo::new(move |_| brushes.with(|b| b.iter().any(Option::is_some)));
    let active = RwSignal::new(None::<usize>);
    let at_x = RwSignal::new(0.5);
    let dragging = StoredValue::new(None::<(usize, f64)>);
    let group_of = move |r: usize| grouped.with(|(_, g)| g.get(r).copied().unwrap_or(0));
    let row_name = move |r: usize| names.with(|n| n.get(r).cloned()).unwrap_or_else(|| format!("Row {}", r + 1));
    let set_brush = move |axis: usize, range: Option<Range>| {
        brushes.update(|b| {
            let n = layout.with_untracked(|l| l.xs.len());
            b.resize(n, None);
            if let Some(slot) = b.get_mut(axis) {
                *slot = range;
            }
        });
    };
    let hover = move |x: f64, y: f64| {
        let next = layout.with_untracked(|l| keep.with_untracked(|k| parallel::nearest(l, k, x, y, 0.06)));
        at_x.set(x);
        if active.get_untracked() != next {
            active.set(next);
        }
    };
    let press = move |ev: leptos::ev::PointerEvent| {
        let Some((x, y, w)) = pointer(&ev) else { return };
        let axis = layout.with_untracked(|l| parallel::axis_near(&l.xs, x, GRAB_PX / w));
        if let Some(a) = axis {
            dragging.set_value(Some((a, y)));
            active.set(None);
            if let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                let _ = el.set_pointer_capture(ev.pointer_id());
            }
        } else {
            hover(x, y);
        }
    };
    let travel = move |ev: leptos::ev::PointerEvent| {
        let Some((x, y, _)) = pointer(&ev) else { return };
        match dragging.get_value() {
            Some((a, start)) => {
                if let Some(range) = parallel::drag(start, y) {
                    set_brush(a, Some(range));
                }
            }
            None => hover(x, y),
        }
    };
    let release = move |ev: leptos::ev::PointerEvent| {
        if let (Some((a, start)), Some((_, y, _))) = (dragging.get_value(), pointer(&ev)) {
            set_brush(a, parallel::drag(start, y));
        }
        dragging.set_value(None);
    };
    let colours = Signal::derive(move || {
        let n = grouped.with(|(g, _)| g.len().clamp(1, 6));
        (0..n).map(plot::colour).chain(std::iter::once("var(--chart-muted)".to_string())).collect::<Vec<_>>()
    });
    let draw = Callback::new(move |f: Frame| {
        let (groups, muted) = f.colours.split_at(f.colours.len().saturating_sub(1));
        let muted = muted.first().copied().unwrap_or([128, 128, 128, 255]);
        layout.with(|l| keep.with(|k| grouped.with(|(_, g)| parallel::raster(l, k, g, (groups, muted), (f.width, f.height), f.dpr))))
    });
    let tip = Signal::derive(move || {
        let r = active.get()?;
        let x = at_x.get();
        let y = layout.with(|l| l.ys.get(r).and_then(|row| parallel::height_at(&l.xs, row, x)))?;
        let mut rows: Vec<TipRow> = Vec::new();
        grouped.with(|(names, g)| {
            if names.len() > 1 {
                let i = g.get(r).copied().unwrap_or(0);
                rows.push(TipRow::new(Some(plot::colour(i)), names.get(i).cloned().unwrap_or_default(), ""));
            }
        });
        dimensions.with(|d| rows.extend(d.iter().map(|d| TipRow::new(None, d.name.clone(), plot::cell(d.values.get(r).copied(), fmt)))));
        Some(Tip { x, y, title: row_name(r), rows })
    });
    let table_head = move || {
        let mut h = vec![String::new()];
        if grouped.with(|(g, _)| g.len() > 1) {
            h.push("Group".into());
        }
        dimensions.with(|d| h.extend(d.iter().map(|d| d.name.clone())));
        h
    };
    let table_row = move |r: usize| {
        let mut row = vec![row_name(r)];
        grouped.with(|(names, g)| {
            if names.len() > 1 {
                row.push(names.get(g.get(r).copied().unwrap_or(0)).cloned().unwrap_or_default());
            }
        });
        dimensions.with(|d| row.extend(d.iter().map(|d| plot::cell(d.values.get(r).copied(), fmt))));
        row
    };
    provide_export(move || std::iter::once(table_head()).chain((0..rows.get_untracked()).map(table_row)).collect());
    let gutter = move || {
        layout.with(|l| {
            let first = l.axes.first().map(|a| a.ticks(fmt).into_iter().map(|t| t.text).collect::<Vec<_>>()).unwrap_or_default();
            format!("--ox-axis: {}px; --ox-height: {height}px;", plot::axis_width(first.iter().map(String::as_str), 28.0))
        })
    };

    view! {
        <figure data-scope="parallel-coordinates-chart" data-part="root" data-raster=move || big.get().then_some("") data-filtered=move || filtered.get().then_some("") style=gutter>
            <div data-part="plot">
                <Show when=move || !big.get()>
                    <svg data-part="lines" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                        {move || {
                            layout.with(|l| {
                                l.ys
                                    .iter()
                                    .enumerate()
                                    .map(|(r, row)| {
                                        let d = parallel::path(&l.xs, row);
                                        view! {
                                            <path
                                                data-part="line"
                                                data-dim=move || keep.with(|k| !k.get(r).copied().unwrap_or(true)).then_some("")
                                                d=d
                                                stroke=plot::colour(group_of(r))
                                                vector-effect="non-scaling-stroke"
                                            ></path>
                                        }
                                    })
                                    .collect_view()
                            })
                        }}
                    </svg>
                </Show>
                <Show when=move || big.get()>
                    <RasterCanvas colours draw />
                </Show>
                <svg data-part="highlight" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                    {move || {
                        let r = active.get()?;
                        let d = layout.with(|l| l.ys.get(r).map(|row| parallel::path(&l.xs, row)))?;
                        Some(view! { <path data-part="active-line" d=d stroke=plot::colour(group_of(r)) vector-effect="non-scaling-stroke"></path> })
                    }}
                </svg>
                {move || {
                    let names: Vec<String> = dimensions.with(|d| d.iter().map(|d| d.name.clone()).collect());
                    let last = names.len().saturating_sub(1);
                    layout
                        .get()
                        .axes
                        .iter()
                        .zip(names)
                        .enumerate()
                        .map(|(a, (axis, name))| {
                            let x = layout.with(|l| l.xs[a]);
                            let edge = if a == 0 { "start" } else if a == last { "end" } else { "" };
                            let ticks = axis
                                .ticks(fmt)
                                .into_iter()
                                .map(|t| view! { <span data-part="tick" style=format!("--p: {};", pct(t.at))>{t.text}</span> })
                                .collect_view();
                            view! {
                                <div data-part="axis" data-edge=edge style=format!("--x: {}; --i: {a};", pct(x))>
                                    <span data-part="axis-name" title=name.clone()>{name.clone()}</span>
                                    {ticks}
                                    {move || {
                                        brushes.with(|b| b.get(a).copied().flatten()).map(|(lo, hi)| {
                                            view! { <span data-part="brush" style=format!("--lo: {}; --hi: {};", pct(lo), pct(hi))></span> }
                                        })
                                    }}
                                </div>
                            }
                        })
                        .collect_view()
                }}
                <div
                    data-part="overlay"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Drag along an axis to filter. Use the left and right arrow keys to read each row.")
                    on:pointerdown=press
                    on:pointermove=travel
                    on:pointerup=release
                    on:pointercancel=move |_| dragging.set_value(None)
                    on:pointerleave=move |_| {
                        if dragging.get_value().is_none() {
                            active.set(None);
                        }
                    }
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = keep.with_untracked(|k| parallel::step(&ev.key(), k, active.get_untracked())) {
                            ev.prevent_default();
                            at_x.set(0.5);
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        tip.get()
                            .map(|t| {
                                let rows = t
                                    .rows
                                    .iter()
                                    .cloned()
                                    .map(|r| {
                                        view! {
                                            <li data-part="tooltip-row">
                                                {r.colour.map(|c| view! { <span data-part="swatch" style=format!("background: {c};")></span> })}
                                                <span data-part="tooltip-name">{r.name}</span>
                                                <span data-part="tooltip-value">{r.value}</span>
                                            </li>
                                        }
                                    })
                                    .collect_view();
                                view! {
                                    <div data-part="tooltip" data-side=t.side() data-vertical=t.vertical() style=format!("left: {}; bottom: {};", pct(t.x), pct(t.y))>
                                        <div data-part="tooltip-label">{t.title.clone()}</div>
                                        <ul>{rows}</ul>
                                    </div>
                                }
                            })
                    }}
                </div>
            </div>
            <div data-part="footer">
                <Show when=move || grouped.with(|(g, _)| g.len() > 1)>
                    <ul data-part="legend" aria-hidden="true">
                        {move || {
                            grouped
                                .get()
                                .0
                                .into_iter()
                                .enumerate()
                                .map(|(i, n)| view! { <li data-part="legend-item"><span data-part="swatch" style=format!("background: {};", plot::colour(i))></span>{n}</li> })
                                .collect_view()
                        }}
                    </ul>
                </Show>
                <span data-part="count" aria-live="polite">
                    {move || if filtered.get() { format!("{} of {} rows", shown.get(), rows.get()) } else { format!("{} rows", rows.get()) }}
                </span>
                <Show when=move || filtered.get()>
                    <button type="button" data-part="clear" on:click=move |_| brushes.set(Vec::new())>"Clear filters"</button>
                </Show>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || tip.with(|t| t.as_ref().map(Tip::announce).unwrap_or_default())}</span>
            <table data-part="sr-table">
                <caption>
                    {label}
                    {move || {
                        let (n, cols) = (rows.get(), dimensions.with(Vec::len) + 1);
                        let listed = raster::table_rows(n, cols);
                        (listed < n).then(|| format!(" (first {listed} of {n} rows)"))
                    }}
                </caption>
                <thead>
                    <tr>{move || table_head().into_iter().map(|h| view! { <th scope="col">{h}</th> }).collect_view()}</tr>
                </thead>
                <tbody>
                    {move || {
                        let n = rows.get();
                        (0..raster::table_rows(n, dimensions.with(Vec::len) + 1))
                            .map(|r| {
                                let mut cells = table_row(r).into_iter();
                                let first = cells.next().unwrap_or_default();
                                view! { <tr><th scope="row">{first}</th>{cells.map(|c| view! { <td>{c}</td> }).collect_view()}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
