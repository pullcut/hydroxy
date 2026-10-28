//! `<WaffleChart>`: a grid of unit cells (10×10 by default), each given to a category
//! in proportion to its value. Counts come from `hydroxy_charts::waffle`. More than six
//! categories fold into "Other".

use leptos::prelude::*;
use hydroxy_charts::{fold_other, format_value, nav, waffle};

pub use hydroxy_charts::Datum;

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
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn WaffleChart(
    /// The categories, in fill order.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Budget by team". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// The whole the grid stands for. Default: the sum of the values. A larger total
    /// leaves the remaining cells empty.
    #[prop(optional, into)]
    total: MaybeProp<f64>,
    /// Cells per row.
    #[prop(default = 10)]
    columns: usize,
    /// Rows of cells.
    #[prop(default = 10)]
    rows: usize,
    /// Formats values for the tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (columns, rows) = (columns.max(1), rows.max(1));
    let cells_n = columns * rows;
    let parts = Memo::new(move |_| data.with(|d| fold_other(d, 6)));
    let folded = Memo::new(move |_| data.with(Vec::len) > 6);
    let counts = Memo::new(move |_| parts.with(|p| waffle::counts(&p.iter().map(|d| d.value).collect::<Vec<_>>(), total.get(), cells_n)));
    let grid = Memo::new(move |_| counts.with(|c| waffle::cells(c, cells_n)));
    let active = RwSignal::new(None::<(usize, usize)>);
    let category = Memo::new(move |_| active.get().and_then(|(r, c)| grid.with(|g| g.get(r * columns + c).copied().flatten())));
    let colour = move |i: usize| slot_colour(i, folded.get(), parts.with(Vec::len));
    #[allow(clippy::cast_precision_loss, reason = "cell counts are tiny")]
    let share = move |n: usize| format!("{}%", format_value(n as f64 / cells_n as f64 * 100.0));
    let describe = move || {
        let (row, col) = active.get()?;
        let Some(k) = category.get() else { return Some(format!("Row {}, column {}: empty", row + 1, col + 1)) };
        let datum = parts.with(|p| p.get(k).cloned())?;
        let count = counts.with(|c| c.get(k).copied().unwrap_or(0));
        Some(format!("{}: {}, {count} of {cells_n} cells", datum.label, fmt(datum.value)))
    };
    let first_cell = move |k: usize| grid.with(|g| g.iter().position(|&x| x == Some(k))).map(|p| (p / columns, p % columns));

    view! {
        <figure data-scope="waffle-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to move between cells.")
                style=format!("--ox-cols: {columns}; --ox-rows: {rows};")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), rows, columns) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    grid.get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, k)| {
                            let at = (i / columns, i % columns);
                            view! {
                                <span
                                    data-part="cell"
                                    data-empty=k.is_none().then_some("")
                                    data-active=move || (k.is_some() && category.get() == k).then_some("")
                                    data-focus=move || (active.get() == Some(at)).then_some("")
                                    style=move || format!("--i: {}; background: {};", i.min(60), k.map_or_else(String::new, colour))
                                    on:pointerenter=move |_| active.set(Some(at))
                                    on:pointerdown=move |_| active.set(Some(at))
                                ></span>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let (row, col) = active.get()?;
                    let k = category.get();
                    let datum = k.and_then(|k| parts.with(|p| p.get(k).cloned()));
                    let count = k.map_or(0, |k| counts.with(|c| c.get(k).copied().unwrap_or(0)));
                    let x = frac(col, columns);
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-side={if x > 0.5 { "left" } else { "right" }}
                                style=format!("left: {}; top: {};", pct(x), pct(frac(row, rows)))
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: {};", k.map_or_else(|| "var(--chart-grid)".into(), colour))></span>
                                    {datum.as_ref().map_or_else(|| "Empty".to_string(), |d| d.label.clone())}
                                </div>
                                <ul>
                                    {datum.map(|d| view! {
                                        <li data-part="tooltip-row">
                                            <span data-part="tooltip-name">"Value"</span>
                                            <span data-part="tooltip-value">{fmt(d.value)}</span>
                                        </li>
                                    })}
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Cells"</span>
                                        <span data-part="tooltip-value">{if k.is_some() { format!("{count} ({})", share(count)) } else { "—".into() }}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <Show when=move || parts.with(|p| p.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        parts
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, d)| {
                                view! {
                                    <li
                                        data-part="legend-item"
                                        data-active=move || (category.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| active.set(first_cell(i))
                                        on:pointerleave=move |_| active.set(None)
                                    >
                                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                        {d.label}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || describe().unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Category"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Cells"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let c = counts.get();
                        parts
                            .get()
                            .into_iter()
                            .zip(c)
                            .map(|(d, n)| {
                                view! {
                                    <tr>
                                        <th scope="row">{d.label}</th>
                                        <td>{fmt(d.value)}</td>
                                        <td>{n}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
