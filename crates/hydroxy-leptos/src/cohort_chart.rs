//! `<CohortChart>`: a retention triangle. Rows are cohorts, columns periods since they
//! started, cells shaded on a sequential ramp. Geometry comes from `hydroxy_charts::cohort`.

use hydroxy_charts::cohort::{self, percent};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::format_value;
use hydroxy_charts::nav::{HINT_CELLS, hinted};
use leptos::prelude::*;

pub use hydroxy_charts::cohort::Cohort;

use crate::callbacks::{select_active, watch_hover};
use crate::chart_frame::provide_export;

/// One row per cohort with its size, then a cell per period it has been through, shaded
/// by the share still active and labelled as a percentage. Hover or touch a cell for the
/// counts; focus the grid and use the arrow keys, Home and End to move, Escape to clear.
#[allow(clippy::cast_precision_loss, reason = "grid sizes are small")]
#[component]
pub fn CohortChart(
    /// The cohorts, oldest first. Each has a size and a count per period so far.
    #[prop(into)]
    cohorts: Signal<Vec<Cohort>>,
    /// One label per period, such as "Month 0", "Month 1"…
    #[prop(into)]
    periods: Signal<Vec<String>>,
    /// What the chart shows, such as "Monthly retention by signup month". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// What a count is of, for the tooltip, such as "customers". Default "people".
    #[prop(default = "people".into(), into)]
    noun: String,
    /// Cell height in px.
    #[prop(default = 32)]
    cell_height: u16,
    /// Formats counts in the size column, tooltip and table.
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
    let layout = Memo::new(move |_| cohorts.with(|c| cohort::layout(c)));
    let active = RwSignal::new(None::<(usize, usize)>);
    let noun = StoredValue::new(noun);
    let name = move |list: &[String], i: usize| list.get(i).cloned().unwrap_or_default();
    let describe = move |(r, c): (usize, usize)| {
        let cell = layout.with(|l| l.at(r, c).copied())?;
        let size = cohorts.with(|all| all.get(r).map_or(0.0, |x| x.size));
        let row = cohorts.with(|all| all.get(r).map(|x| x.name.clone()).unwrap_or_default());
        let col = periods.with(|p| name(p, c));
        Some((
            format!("{row} · {col}"),
            format!("{} of {} {} ({})", fmt(cell.count), fmt(size), noun.get_value(), percent(cell.rate)),
        ))
    };
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let (r, c) = active.get()?;
            let count = layout.with(|l| l.at(r, c).map(|x| x.count))?;
            let (row, col) = (cohorts.with(|all| all.get(r).map(|x| x.name.clone()))?, periods.with(|p| name(p, c)));
            Some(EventDatum { series: r, ..EventDatum::category(c, format!("{row} · {col}"), count) })
        })
    });
    watch_hover(datum, on_hover);
    provide_export(move || {
        let head =
            ["Cohort".to_string(), "Size".to_string()].into_iter().chain(periods.get_untracked()).collect::<Vec<_>>();
        let body = cohorts.with_untracked(|all| {
            all.iter()
                .map(|c| {
                    [c.name.clone(), fmt(c.size)]
                        .into_iter()
                        .chain(c.retained.iter().map(|&v| if v.is_finite() { fmt(v) } else { String::new() }))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        });
        std::iter::once(head).chain(body).collect()
    });

    view! {
        <figure data-scope="cohort-chart" data-part="root">
            <div
                data-part="frame"
                style=move || layout.with(|l| format!("--cols: {}; --rows: {}; --cell: {cell_height}px;", l.cols.max(1), l.rows.max(1)))
            >
                <div data-part="corner" aria-hidden="true">
                    <span data-part="label">"Cohort"</span>
                    <span data-part="label" data-size="">"Size"</span>
                </div>
                <div data-part="col-labels" aria-hidden="true">
                    {move || periods.get().into_iter().map(|p| { let t = p.clone(); view! { <span data-part="label" title=t>{p}</span> } }).collect_view()}
                </div>
                <div data-part="row-labels" aria-hidden="true">
                    {move || {
                        cohorts
                            .get()
                            .into_iter()
                            .map(|c| {
                                let t = c.name.clone();
                                view! {
                                    <span data-part="row-label" title=t>
                                        <span data-part="label">{c.name}</span>
                                        <span data-part="size">{fmt(c.size)}</span>
                                    </span>
                                }
                            })
                            .collect_view()
                    }}
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
                        if let Some(next) = layout.with(|l| cohort::key(l, &ev.key(), active.get_untracked())) {
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
                                        data-strong=(cell.level > 0.55).then_some("")
                                        data-active=move || (active.get() == Some(at)).then_some("")
                                        style=format!(
                                            "grid-row: {}; grid-column: {}; --level: {:.3}; --i: {}; --ox-n: {n};",
                                            cell.row + 1,
                                            cell.col + 1,
                                            cell.level,
                                            cell.row + cell.col,
                                        )
                                        on:pointerenter=move |_| active.set(Some(at))
                                        on:pointerdown=move |_| active.set(Some(at))
                                        on:click=move |_| {
                                            active.set(Some(at));
                                            select_active(datum, on_select);
                                        }
                                    >
                                        {percent(cell.rate)}
                                    </span>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let (r, c) = active.get()?;
                        let (title, value) = describe((r, c))?;
                        let (rows, cols) = layout.with(|l| (l.rows.max(1) as f64, l.cols.max(1) as f64));
                        let below = r == 0;
                        let edge = if (c as f64) < cols / 3.0 { "start" } else if (c as f64) >= cols * 2.0 / 3.0 { "end" } else { "" };
                        Some(view! {
                            <div
                                data-part="tooltip"
                                data-side={if below { "below" } else { "above" }}
                                data-edge=edge
                                style=format!(
                                    "left: {:.3}%; top: {:.3}%;",
                                    (c as f64 + 0.5) / cols * 100.0,
                                    (r as f64 + if below { 1.0 } else { 0.0 }) / rows * 100.0,
                                )
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">{title}</div>
                                <div data-part="tooltip-value">{value}</div>
                            </div>
                        })
                    }}
                </div>
            </div>
            <div data-part="scale" aria-hidden="true">
                <span data-part="scale-value">"0%"</span>
                <span data-part="ramp"></span>
                <span data-part="scale-value">
                    {move || layout.with(|l| percent(l.cells.iter().filter(|c| c.col > 0).map(|c| c.rate).fold(0.0, f64::max)))}
                    "+"
                </span>
            </div>
            <span data-part="sr-live" aria-live="polite">
                {move || active.get().and_then(describe).map(|(t, v)| format!("{t}: {v}")).unwrap_or_default()}
            </span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Cohort"</th>
                        <th scope="col">"Size"</th>
                        {move || periods.get().into_iter().map(|p| view! { <th scope="col">{p}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        cohorts
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(r, c)| {
                                let cells = (0..l.cols)
                                    .map(|col| view! { <td>{l.at(r, col).map_or_else(|| "–".to_string(), |x| percent(x.rate))}</td> })
                                    .collect_view();
                                view! { <tr><th scope="row">{c.name}</th><td>{fmt(c.size)}</td>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
