//! `<BarChart>`: one series of vertical bars. Geometry comes from `oxide_charts::bar`;
//! layout is plain CSS, so it fills its container without measuring.

use leptos::prelude::*;
use oxide_charts::{bar, format_value};

pub use oxide_charts::Datum;
pub use oxide_charts::bar::Highlight;

/// A single-series bar chart: grey bars with one picked out in the accent colour.
#[component]
pub fn BarChart(
    /// The values, in display order.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Interviews per week". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the category labels.
    #[prop(default = 180)]
    height: u16,
    /// Which bar gets the accent colour. Default: the last.
    #[prop(optional)]
    highlight: Highlight,
    /// Show every value above its bar, not only the highlighted one and the hovered one.
    #[prop(optional)]
    show_values: bool,
    /// Formats values for the axis, labels and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        data.with(|d| {
            let values: Vec<f64> = d.iter().map(|x| x.value).collect();
            bar::layout(&values, highlight.resolve(values.len()), 5)
        })
    });
    let pct = |f: f64| format!("{:.3}%", f * 100.0);
    // Bars are keyed by label: a changed value glides its own bar; only new bars grow in.
    let rows = move || data.get().into_iter().enumerate().collect::<Vec<_>>();

    view! {
        <figure data-scope="bar-chart" data-part="root" data-show-values=show_values.then_some("")>
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
                                        data-baseline=(g.fraction == 0.0).then_some("")
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
                    <For
                        each=rows
                        key=|(_, d)| d.label.clone()
                        children=move |(i, d)| {
                            let label = d.label;
                            let pos = Memo::new(move |_| data.with(|ds| ds.iter().position(|x| x.label == label)));
                            let bar = Memo::new(move |_| {
                                pos.get()
                                    .and_then(|p| layout.with(|l| l.bars.get(p).copied()))
                                    .unwrap_or(bar::Bar { fraction: 0.0, highlighted: false })
                            });
                            let value = Memo::new(move |_| pos.get().and_then(|p| data.with(|ds| ds.get(p).map(|x| x.value))).unwrap_or(0.0));
                            view! {
                                <div data-part="column" style=format!("--i: {i};") data-highlighted=move || bar.get().highlighted.then_some("")>
                                    <span data-part="value" style=move || format!("bottom: {};", pct(bar.get().fraction))>
                                        {move || fmt(value.get())}
                                    </span>
                                    <div data-part="bar" style=move || format!("height: {};", pct(bar.get().fraction))></div>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
            <div data-part="labels" aria-hidden="true">
                {move || {
                    data.get()
                        .into_iter()
                        .map(|d| {
                            let title = d.label.clone();
                            view! { <span data-part="label" title=title>{d.label}</span> }
                        })
                        .collect_view()
                }}
            </div>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <tbody>
                    {move || {
                        data.get()
                            .into_iter()
                            .map(|d| view! { <tr><th scope="row">{d.label}</th><td>{fmt(d.value)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
