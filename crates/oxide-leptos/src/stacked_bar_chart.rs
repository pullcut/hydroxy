//! `<StackedBarChart>`: several series stacked per category, with totals and a legend.
//! Geometry comes from `oxide_charts::stacked`; layout is plain CSS. Negative values
//! stack down from the zero line, and the total shown is the net sum.

use leptos::prelude::*;
use oxide_charts::{format_value, stacked};

pub use oxide_charts::line::Series;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Vertical stacked bars: one column per label, one segment per series.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn StackedBarChart(
    /// The series, stacked bottom-up in order, in categorical colours. Up to six read well.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One label per column.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows, such as "Tickets by channel per week". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the labels and legend.
    #[prop(default = 200)]
    height: u16,
    /// Show each column's total above it.
    #[prop(default = true)]
    show_totals: bool,
    /// Formats values for the axis, totals and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        let len = labels.with(Vec::len);
        series.with(|s| {
            let values: Vec<Vec<f64>> = s.iter().map(|s| s.values.clone()).collect();
            stacked::layout(&values, len, 5)
        })
    });
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).unwrap_or(0.0);

    view! {
        <figure data-scope="stacked-bar-chart" data-part="root" data-show-totals=show_totals.then_some("")>
            <div data-part="plot" style=format!("height: {height}px;") aria-hidden="true">
                <div data-part="grid">
                    {move || {
                        layout
                            .get()
                            .gridlines
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div
                                        data-part="gridline"
                                        data-baseline=(g.value == 0.0).then_some("")
                                        style=format!("bottom: {};", pct(g.fraction))
                                    >
                                        <span data-part="tick">{fmt(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div data-part="bars">
                    {move || {
                        let l = layout.get();
                        let s = series.get();
                        let names = labels.get();
                        l.columns
                            .into_iter()
                            .enumerate()
                            .map(|(i, col)| {
                                let segments = col
                                    .segments
                                    .iter()
                                    .zip(&s)
                                    .enumerate()
                                    .filter(|(_, (seg, _))| seg.height > 0.0)
                                    .map(|(si, (seg, ser))| {
                                        let title = format!("{}, {}: {}", names.get(i).cloned().unwrap_or_default(), ser.name, fmt(value_at(ser, i)));
                                        view! {
                                            <div
                                                data-part="segment"
                                                data-index=si
                                                title=title
                                                style=format!(
                                                    "bottom: {}; height: {}; background: {};",
                                                    pct(seg.bottom),
                                                    pct(seg.height),
                                                    colour(si),
                                                )
                                            ></div>
                                        }
                                    })
                                    .collect_view();
                                view! {
                                    <div data-part="column" style=format!("--i: {i}; --ox-baseline: {};", pct(l.baseline))>
                                        <div data-part="stack">{segments}</div>
                                        <span data-part="total" style=format!("bottom: {};", pct(col.top))>{fmt(col.total)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
            <div data-part="labels" aria-hidden="true">
                {move || {
                    labels
                        .get()
                        .into_iter()
                        .map(|l| {
                            let title = l.clone();
                            view! { <span data-part="label" title=title>{l}</span> }
                        })
                        .collect_view()
                }}
            </div>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    series
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, s)| {
                            view! {
                                <li data-part="legend-item" data-index=i>
                                    <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                    {s.name}
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <td></td>
                        {move || series.get().into_iter().map(|s| view! { <th scope="col">{s.name}</th> }).collect_view()}
                        <th scope="col">"Total"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let s = series.get();
                        let l = layout.get();
                        labels
                            .get()
                            .into_iter()
                            .zip(l.columns)
                            .enumerate()
                            .map(|(i, (name, col))| {
                                let cells = s.iter().map(|s| view! { <td>{fmt(value_at(s, i))}</td> }).collect_view();
                                view! { <tr><th scope="row">{name}</th>{cells}<td>{fmt(col.total)}</td></tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
