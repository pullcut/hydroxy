//! `<BarChart>`: one series of vertical bars. Geometry comes from `hydroxy_charts::bar`;
//! layout is plain CSS, so it fills its container without measuring.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{annotate, bar, format_value, nav};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::line_chart::annotation_views;
use crate::plot_kit::pct;
pub use hydroxy_charts::Datum;
pub use hydroxy_charts::Orientation;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::bar::Highlight;

/// Which way an edge tooltip leans so it stays inside the plot: the first third of the
/// marks "start", the last third "end".
pub(crate) fn edge_side(i: usize, len: usize) -> &'static str {
    if i * 3 < len {
        "start"
    } else if i * 3 >= len * 2 {
        "end"
    } else {
        ""
    }
}

/// A single-series bar chart: grey bars with one picked out in the accent colour.
/// Hover a bar, or focus the plot and use the arrow keys, to read its value.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
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
    /// Vertical (default) or horizontal bars. Horizontal suits long category names.
    #[prop(optional)]
    orientation: Orientation,
    /// Show every value above its bar, not only the highlighted one and the hovered one.
    #[prop(optional)]
    show_values: bool,
    /// Formats values for the axis, labels and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` values are category indices; `y`
    /// values are on the value axis, which grows to fit them. Horizontal bars turn them round.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// A value range the axis must cover, such as the shared domain of small multiples.
    #[prop(optional, into)]
    y_domain: MaybeProp<(f64, f64)>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        let extent =
            hydroxy_charts::small_multiples::widen(annotations.with(|a| annotate::y_extent(a)), y_domain.get());
        data.with(|d| {
            let values: Vec<f64> = d.iter().map(|x| x.value).collect();
            bar::layout_with_extent(&values, highlight.resolve(values.len()), 5, extent)
        })
    });
    let horizontal = orientation == Orientation::Horizontal;
    let placed = Memo::new(move |_| {
        let (n, max) = (data.with(Vec::len), layout.with(|l| l.max));
        let placed = annotations.with(|a| annotate::place(a, |i| bar::category_centre(i, n), |v| v / max));
        if horizontal { placed.into_iter().map(bar::transpose).collect() } else { placed }
    });
    // Horizontal bars take their size from `--p` in CSS; vertical ones keep inline `height`/`bottom`.
    let place = move |prop: &str, f: f64| {
        if horizontal { format!("--p: {};", pct(f)) } else { format!("{prop}: {}; --p: {};", pct(f), pct(f)) }
    };
    // Bars are keyed by label: a changed value glides its own bar; only new bars grow in.
    let rows = move || data.get().into_iter().enumerate().collect::<Vec<_>>();
    let active = RwSignal::new(None::<usize>);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            data.with(|d| d.get(i).map(|d| EventDatum::category(i, d.label.clone(), d.value)))
        })
    });
    watch_hover(datum, on_hover);
    let name = label.clone();
    let announcement = move || {
        active
            .get()
            .and_then(|i| data.with(|d| d.get(i).map(|d| format!("{}: {}", d.label, fmt(d.value)))))
            .unwrap_or_default()
    };

    view! {
        <figure data-scope="bar-chart" data-part="root" data-orientation=orientation.attr() data-show-values=show_values.then_some("")>
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
                                        style=place("bottom", g.fraction)
                                    >
                                        <span data-part="tick">{fmt(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div data-part="annotations" aria-hidden="true">{move || annotation_views(placed.get())}</div>
                <div
                    data-part="bars"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to read each bar.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            select_active(datum, on_select);
                        }
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), data.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
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
                            let is_active = move || pos.get().is_some_and(|p| active.get() == Some(p));
                            let name = name.clone();
                            let tip = move || {
                                is_active()
                                    .then(|| {
                                        let p = pos.get().unwrap_or(0);
                                        let category = data.with(|ds| ds.get(p).map(|d| d.label.clone()).unwrap_or_default());
                                        view! {
                                            <div
                                                data-part="tooltip"
                                                data-side=edge_side(p, data.with(Vec::len))
                                                style=place("bottom", bar.get().fraction)
                                            >
                                                <div data-part="tooltip-label">{category}</div>
                                                <ul>
                                                    <li data-part="tooltip-row">
                                                        <span data-part="swatch"></span>
                                                        <span data-part="tooltip-name">{name.clone()}</span>
                                                        <span data-part="tooltip-value">{fmt(value.get())}</span>
                                                    </li>
                                                </ul>
                                            </div>
                                        }
                                    })
                            };
                            view! {
                                <div
                                    data-part="column"
                                    style=format!("--i: {i};")
                                    aria-hidden="true"
                                    data-highlighted=move || bar.get().highlighted.then_some("")
                                    data-active=move || is_active().then_some("")
                                    on:pointerenter=move |_| active.set(pos.get_untracked())
                                    on:click=move |_| {
                                        active.set(pos.get_untracked());
                                        select_active(datum, on_select);
                                    }
                                >
                                    <span data-part="value" style=move || place("bottom", bar.get().fraction)>
                                        {move || fmt(value.get())}
                                    </span>
                                    <div data-part="bar" style=move || place("height", bar.get().fraction)></div>
                                    {tip}
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
            <span data-part="sr-live" aria-live="polite">{announcement}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <td></td>
                        <th scope="col">{label}</th>
                    </tr>
                </thead>
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
