//! `<MarimekkoChart>`: variable-width 100% stacks. Each column is as wide as its
//! category's share of the grand total; its segments split it by series. Geometry
//! comes from `hydroxy_charts::marimekko`.

use hydroxy_charts::{format_value, marimekko, nav};
use leptos::prelude::*;

pub use hydroxy_charts::line::Series;

/// Series colours in categorical order; a seventh series and beyond are grey, never a repeat.
fn colour(i: usize) -> String {
    if i < 6 { format!("var(--chart-{})", i + 1) } else { "var(--chart-muted)".into() }
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

fn percent(f: f64) -> String {
    format!("{}%", format_value(f * 100.0))
}

/// A Marimekko chart. Hover, touch or focus a segment to read it; focus the chart and
/// use Left and Right to move between categories, Up and Down between series, Home
/// and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn MarimekkoChart(
    /// The series, stacked from the bottom, each with one value per category.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// Category names, one per column, left to right.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows, such as "Market by region and segment". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px.
    #[prop(default = 280)]
    height: u16,
    /// Formats values for the column totals, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let columns = Memo::new(move |_| {
        let n = labels.with(Vec::len);
        series.with(|s| marimekko::layout(&s.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), n))
    });
    // Grid rows run top to bottom, so row 0 is the last (top) series.
    let active = RwSignal::new(None::<(usize, usize)>);
    let n_series = move || series.with(Vec::len);
    let pick = move || active.get().map(|(r, c)| (n_series().saturating_sub(r + 1), c));
    let value =
        move |s: usize, c: usize| series.with(|x| x.get(s).and_then(|x| x.values.get(c).copied()).unwrap_or(0.0));
    let grand = move || columns.with(|c| c.iter().map(|c| c.total).sum::<f64>());
    let describe = move || {
        let (s, c) = pick()?;
        let name = series.with(|x| x.get(s).map(|x| x.name.clone()))?;
        let cat = labels.with(|l| l.get(c).cloned())?;
        let share = columns.with(|cols| cols.get(c).and_then(|col| col.segments.get(s).map(|g| g.share)))?;
        Some(format!("{cat}, {name}: {}, {} of {cat}", fmt(value(s, c)), percent(share)))
    };
    let set = move |s: usize, c: usize| active.set(Some((n_series().saturating_sub(s + 1), c)));

    view! {
        <figure data-scope="marimekko-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to move between segments.")
                style=format!("height: {height}px;")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let cols = labels.with(Vec::len);
                    if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), n_series(), cols) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <div data-part="grid" aria-hidden="true">
                    {[0.0, 0.25, 0.5, 0.75, 1.0]
                        .into_iter()
                        .map(|f| {
                            view! {
                                <div data-part="gridline" style=format!("bottom: {};", pct(f))>
                                    <span data-part="tick">{percent(f)}</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                {move || {
                    columns
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(c, col)| {
                            view! {
                                <div data-part="column" style=format!("left: {}; width: {}; --i: {c};", pct(col.x0), pct(col.x1 - col.x0))>
                                    {col
                                        .segments
                                        .iter()
                                        .enumerate()
                                        .map(|(s, g)| {
                                            view! {
                                                <span
                                                    data-part="segment"
                                                    data-series=s
                                                    data-active=move || (pick() == Some((s, c))).then_some("")
                                                    style=format!("bottom: {}; height: {}; background: {};", pct(g.y0), pct(g.y1 - g.y0), colour(s))
                                                    on:pointerenter=move |_| set(s, c)
                                                    on:pointerdown=move |_| set(s, c)
                                                ></span>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let (s, c) = pick()?;
                    let col = columns.with(|cols| cols.get(c).cloned())?;
                    let seg = col.segments.get(s).copied()?;
                    let x = f64::midpoint(col.x0, col.x1);
                    let total = grand();
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-side={if x > 0.5 { "left" } else { "right" }}
                                style=format!("left: {}; top: {};", pct(x), pct(1.0 - f64::midpoint(seg.y0, seg.y1)))
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: {};", colour(s))></span>
                                    {format!(
                                        "{}, {}",
                                        labels.with(|l| l.get(c).cloned()).unwrap_or_default(),
                                        series.with(|x| x[s].name.clone()),
                                    )}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Value"</span>
                                        <span data-part="tooltip-value">{fmt(value(s, c))}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Of column"</span>
                                        <span data-part="tooltip-value">{percent(seg.share)}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Of total"</span>
                                        <span data-part="tooltip-value">{percent(if total > 0.0 { value(s, c).max(0.0) / total } else { 0.0 })}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <div data-part="x-labels" aria-hidden="true">
                {move || {
                    columns
                        .get()
                        .into_iter()
                        .zip(labels.get())
                        .map(|(col, name)| {
                            view! {
                                <span data-part="x-label" style=format!("left: {}; max-width: {};", pct(f64::midpoint(col.x0, col.x1)), pct(col.x1 - col.x0))>
                                    <span data-part="x-name">{name}</span>
                                    <span data-part="x-total">{fmt(col.total)}</span>
                                </span>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <Show when=move || series.with(|s| s.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        series
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, s)| {
                                view! {
                                    <li data-part="legend-item" data-active=move || (pick().map(|p| p.0) == Some(i)).then_some("")>
                                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                        {s.name}
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
                        {move || series.get().into_iter().map(|s| view! { <th scope="col">{s.name}</th> }).collect_view()}
                        <th scope="col">"Total"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cols = columns.get();
                        labels
                            .get()
                            .into_iter()
                            .zip(cols)
                            .enumerate()
                            .map(|(c, (name, col))| {
                                view! {
                                    <tr>
                                        <th scope="row">{name}</th>
                                        {(0..n_series()).map(|s| view! { <td>{fmt(value(s, c))}</td> }).collect_view()}
                                        <td>{fmt(col.total)}</td>
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
