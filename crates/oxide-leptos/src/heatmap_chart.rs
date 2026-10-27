//! `<HeatmapChart>`: a rows × columns grid whose cells shade by value. The scale is one
//! chart colour mixed into the surface, so it follows the theme. Geometry comes from
//! `oxide_charts::heatmap`.

use leptos::prelude::*;
use oxide_charts::{format_value, heatmap, nav};

/// A grid of values, one `Vec` per row, shaded from the lowest to the highest value.
/// Hover or touch a cell to read it; focus the grid and use the arrow keys, Home and End to move,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "grid sizes are tiny")]
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
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| values.with(|v| heatmap::layout(v)));
    let active = RwSignal::new(None::<(usize, usize)>);
    let name = move |list: Signal<Vec<String>>, i: usize| list.with(|l| l.get(i).cloned().unwrap_or_default());
    let describe = move |(r, c): (usize, usize)| {
        let v = layout.with(|l| l.cells.get(r * l.cols + c).and_then(|cell| cell.value));
        format!("{}, {}: {}", name(rows, r), name(columns, c), v.map_or_else(|| "no data".to_string(), fmt))
    };

    view! {
        <figure data-scope="heatmap-chart" data-part="root">
            <div
                data-part="frame"
                style=move || {
                    let l = layout.get();
                    format!("--cols: {}; --rows: {}; --cell: {cell_height}px;", l.cols.max(1), l.rows.max(1))
                }
            >
                <div data-part="col-labels" aria-hidden="true">
                    {move || columns.get().into_iter().map(|c| { let t = c.clone(); view! { <span data-part="label" title=t>{c}</span> } }).collect_view()}
                </div>
                <div data-part="row-labels" aria-hidden="true">
                    {move || rows.get().into_iter().map(|r| { let t = r.clone(); view! { <span data-part="label" title=t>{r}</span> } }).collect_view()}
                </div>
                <div
                    data-part="cells"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to move between cells.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let (r, c) = layout.with(|l| (l.rows, l.cols));
                        if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), r, c) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
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
                            .collect_view()
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
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Row"</th>
                        {move || columns.get().into_iter().map(|c| view! { <th scope="col">{c}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        rows.get()
                            .into_iter()
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
