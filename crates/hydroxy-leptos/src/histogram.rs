//! `<Histogram>`: how values are distributed, as adjacent equal-width bins.
//! Binning and heights come from `hydroxy_charts::histogram`; layout is plain CSS.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{annotate, format_value, histogram, line, nav};
use leptos::prelude::*;

use crate::bar_chart::edge_side;
use crate::line_chart::annotation_views;
use crate::plot_kit::pct;
pub use hydroxy_charts::annotate::Annotation;

/// A histogram of raw values. Bins use Sturges' rule unless `bins` is set. Hover a bin, or
/// focus the plot and use the arrow keys, to read its range and count.
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
    /// Reference lines, bands and labelled points. `x` values are on the value axis (such as
    /// a target time), `y` values are counts. The count axis grows to fit them.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Called when the pointer or keyboard moves to a bin, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a bin.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let binned = Memo::new(move |_| values.with(|v| histogram::bins(v, bins)));
    let layout = Memo::new(move |_| {
        let extent = annotations.with(|a| annotate::y_extent(a));
        binned.with(|b| histogram::layout_with_extent(b, 5, extent))
    });
    let placed = Memo::new(move |_| {
        let max = layout.with(|l| l.max);
        binned.with(|b| annotations.with(|a| annotate::place(a, |x| histogram::x_fraction(b, x), |v| v / max)))
    });
    let range = move |b: &histogram::Bin| format!("{} to {}", fmt(b.lo), fmt(b.hi));
    let active = RwSignal::new(None::<usize>);
    let datum = Signal::derive(move || {
        let i = active.get()?;
        binned.with(|b| {
            let bin = b.get(i)?;
            #[allow(clippy::cast_precision_loss, reason = "counts are small")]
            let count = bin.count as f64;
            Some(EventDatum::category(i, range(bin), count))
        })
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let announcement = move || {
        active
            .get()
            .and_then(|i| binned.with(|b| b.get(i).map(|b| format!("{}: {}", range(b), b.count))))
            .unwrap_or_default()
    };

    view! {
        <figure data-scope="histogram" data-part="root">
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
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
                <div data-part="annotations" aria-hidden="true">{move || annotation_views(placed.get())}</div>
                <div
                    data-part="columns"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to read each bin.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), binned.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let all = binned.get();
                        let n = all.len();
                        all.into_iter()
                            .zip(l.bars)
                            .enumerate()
                            .map(|(i, (b, bar))| {
                                let tip = move || {
                                    (active.get() == Some(i))
                                        .then(|| {
                                            view! {
                                                <div data-part="tooltip" data-side=edge_side(i, n) style=format!("bottom: {};", pct(bar.fraction))>
                                                    <div data-part="tooltip-label">{range(&b)}</div>
                                                    <ul>
                                                        <li data-part="tooltip-row">
                                                            <span data-part="swatch"></span>
                                                            <span data-part="tooltip-name">"Count"</span>
                                                            <span data-part="tooltip-value">{b.count}</span>
                                                        </li>
                                                    </ul>
                                                </div>
                                            }
                                        })
                                };
                                view! {
                                    <div
                                        data-part="column"
                                        aria-hidden="true"
                                        style=format!("--i: {i};")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| active.set(Some(i))
                                    >
                                        <span data-part="count" style=format!("bottom: {};", pct(bar.fraction))>{b.count}</span>
                                        <div data-part="bar" style=format!("height: {};", pct(bar.fraction))></div>
                                        {tip}
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
            <span data-part="sr-live" aria-live="polite">{announcement}</span>
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
