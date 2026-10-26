//! `<Histogram>`: how values are distributed, as adjacent equal-width bins.
//! Binning and heights come from `oxide_charts::histogram`; layout is plain CSS.

use leptos::prelude::*;
use oxide_charts::{format_value, histogram, line};

/// A histogram of raw values. Bins use Sturges' rule unless `bins` is set.
#[component]
pub fn Histogram(
    /// The raw observations. Non-finite values are ignored.
    #[prop(into)]
    values: Signal<Vec<f64>>,
    /// What the chart shows, such as "Response time". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Explicit number of bins. Default: Sturges' rule, `ceil(log2 n) + 1`.
    #[prop(optional)]
    bins: Option<usize>,
    /// Plot height in px, excluding the axis labels.
    #[prop(default = 200)]
    height: u16,
    /// Formats bin edges for the axis and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let binned = Memo::new(move |_| values.with(|v| histogram::bins(v, bins)));
    let layout = Memo::new(move |_| binned.with(|b| histogram::layout(b, 5)));
    let pct = |f: f64| format!("{:.3}%", f * 100.0);
    let range = move |b: &histogram::Bin| format!("{} to {}", fmt(b.lo), fmt(b.hi));

    view! {
        <figure data-scope="histogram" data-part="root">
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
                                        <span data-part="tick">{format_value(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div data-part="columns">
                    {move || {
                        let l = layout.get();
                        binned
                            .get()
                            .into_iter()
                            .zip(l.bars)
                            .enumerate()
                            .map(|(i, (b, bar))| {
                                view! {
                                    <div data-part="column" style=format!("--i: {i};") title=format!("{}: {}", range(&b), b.count)>
                                        <span data-part="count" style=format!("bottom: {};", pct(bar.fraction))>{b.count}</span>
                                        <div data-part="bar" style=format!("height: {};", pct(bar.fraction))></div>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
            <div data-part="labels" aria-hidden="true">
                {move || {
                    let b = binned.get();
                    let n = b.len();
                    // One label per bin edge: n + 1 of them, thinned to fit.
                    line::label_indices(n + 1, 7)
                        .into_iter()
                        .filter(|_| n > 0)
                        .map(|i| {
                            let edge = b.get(i).map_or_else(|| b[n - 1].hi, |bin| bin.lo);
                            #[allow(clippy::cast_precision_loss, reason = "bin counts are small")]
                            let at = i as f64 / n as f64;
                            view! { <span data-part="label" style=format!("left: {};", pct(at))>{fmt(edge)}</span> }
                        })
                        .collect_view()
                }}
            </div>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Range"</th>
                        <th scope="col">"Count"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        binned
                            .get()
                            .into_iter()
                            .map(|b| view! { <tr><th scope="row">{range(&b)}</th><td>{b.count}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
