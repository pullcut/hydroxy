//! `<StackedBarChart>`: several series stacked per category, with totals and a legend.
//! Geometry comes from `hydroxy_charts::stacked`; layout is plain CSS. Negative values
//! stack down from the zero line, and the total shown is the net sum.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::scale::Linear;
use hydroxy_charts::{annotate, bar, format_value, nav, stacked};
use leptos::prelude::*;

use crate::bar_chart::edge_side;
use crate::line_chart::annotation_views;
use crate::plot_kit::{colour, pct};
pub use hydroxy_charts::Orientation;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;

/// Stacked bars: one column per label, one segment per series. Hover a column, or focus the
/// plot and use the arrow keys, to read every series in it.
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
    /// Vertical (default) or horizontal stacks. Horizontal suits long category names.
    #[prop(optional)]
    orientation: Orientation,
    /// Show each column's total above it.
    #[prop(default = true)]
    show_totals: bool,
    /// Formats values for the axis, totals and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Stretch every column to 100% so the stacks compare shares. The axis and tooltips
    /// read in percent; totals and the table keep the real values.
    #[prop(optional)]
    normalize: bool,
    /// Reference lines, bands and labelled points. `x` values are column indices; `y`
    /// values are on the value axis (percent when normalized), which grows to fit them.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Called when the hovered or focused column changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a column.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let horizontal = orientation == Orientation::Horizontal;
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let tick = move |v: f64| if normalize { format!("{}%", format_value(v)) } else { fmt(v) };
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
    // The stacked values: real ones, or each column's shares when normalized.
    let drawn = Memo::new(move |_| {
        let len = labels.with(Vec::len);
        series.with(|s| {
            let values: Vec<Vec<f64>> = s.iter().map(|s| s.values.clone()).collect();
            if normalize { stacked::normalize(&values, len) } else { values }
        })
    });
    let layout = Memo::new(move |_| {
        let extent = annotations.with(|a| annotate::y_extent(a));
        drawn.with(|v| stacked::layout_with_extent(v, labels.with(Vec::len), 5, extent))
    });
    let totals = move |i: usize| series.with(|s| s.iter().map(|s| value_at(s, i)).sum::<f64>());
    let placed = Memo::new(move |_| {
        let (n, (min, max)) = (labels.with(Vec::len), layout.with(|l| (l.min, l.max)));
        let y = Linear::new((min, max), (0.0, 1.0));
        let placed = annotations.with(|a| annotate::place(a, |i| bar::category_centre(i, n), |v| y.map(v)));
        if horizontal { placed.into_iter().map(bar::transpose).collect() } else { placed }
    });
    let active = RwSignal::new(None::<usize>);
    // "4 (25%)" when normalized, else just the value.
    let reading = move |si: usize, i: usize| {
        let v = series.with(|s| s.get(si).map_or(0.0, |s| value_at(s, i)));
        let share = drawn.with(|d| d.get(si).and_then(|d| d.get(i)).copied().unwrap_or(0.0));
        if normalize { format!("{} ({}%)", fmt(v), format_value(share)) } else { fmt(v) }
    };
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let name = labels.with(|l| l.get(i).cloned())?;
        Some(EventDatum::category(i, name, totals(i)))
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let announcement = move || {
        active.get().map_or_else(String::new, |i| {
            let name = labels.with(|l| l.get(i).cloned().unwrap_or_default());
            let parts: Vec<String> =
                series.with(|s| s.iter().enumerate().map(|(si, s)| format!("{} {}", s.name, reading(si, i))).collect());
            format!("{name}: {}, total {}", parts.join(", "), fmt(totals(i)))
        })
    };

    view! {
        <figure
            data-scope="stacked-bar-chart"
            data-part="root"
            data-orientation=orientation.attr()
            data-show-totals=show_totals.then_some("")
            data-normalized=normalize.then_some("")
        >
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
                                        data-baseline=(g.value == 0.0).then_some("")
                                        style=if horizontal { format!("--p: {};", pct(g.fraction)) } else { format!("bottom: {};", pct(g.fraction)) }
                                    >
                                        <span data-part="tick">{tick(g.value)}</span>
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
                    aria-label=format!("{label}. Use the arrow keys to read each column.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), labels.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let s = series.get();
                        let names = labels.get();
                        let n = names.len();
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
                                    .map(|(si, (seg, _))| {
                                        view! {
                                            <div
                                                data-part="segment"
                                                data-index=si
                                                data-end=col.end(si, l.baseline)
                                                style=if horizontal {
                                                    format!("--b: {}; --h: {}; background: {};", pct(seg.bottom), pct(seg.height), colour(si))
                                                } else {
                                                    format!("bottom: {}; height: {}; background: {};", pct(seg.bottom), pct(seg.height), colour(si))
                                                }
                                            ></div>
                                        }
                                    })
                                    .collect_view();
                                let at = if horizontal { format!("--p: {};", pct(col.top)) } else { format!("bottom: {};", pct(col.top)) };
                                let tip_at = at.clone();
                                let category = names.get(i).cloned().unwrap_or_default();
                                let rows = s.clone();
                                let tip = move || {
                                    (active.get() == Some(i))
                                        .then(|| {
                                            let rows = rows
                                                .iter()
                                                .enumerate()
                                                .map(|(si, ser)| {
                                                    view! {
                                                        <li data-part="tooltip-row">
                                                            <span data-part="swatch" style=format!("background: {};", colour(si))></span>
                                                            <span data-part="tooltip-name">{ser.name.clone()}</span>
                                                            <span data-part="tooltip-value">{reading(si, i)}</span>
                                                        </li>
                                                    }
                                                })
                                                .collect_view();
                                            view! {
                                                <div data-part="tooltip" data-side=edge_side(i, n) style=tip_at.clone()>
                                                    <div data-part="tooltip-label">{category.clone()}</div>
                                                    <ul>{rows}</ul>
                                                </div>
                                            }
                                        })
                                };
                                view! {
                                    <div
                                        data-part="column"
                                        aria-hidden="true"
                                        style=format!("--i: {i}; --ox-baseline: {};", pct(l.baseline))
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| active.set(Some(i))
                                    >
                                        <div data-part="stack">{segments}</div>
                                        <span data-part="total" style=at>{fmt(totals(i))}</span>
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
            <span data-part="sr-live" aria-live="polite">{announcement}</span>
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
                        labels
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, name)| {
                                let cells = s.iter().map(|s| view! { <td>{fmt(value_at(s, i))}</td> }).collect_view();
                                view! { <tr><th scope="row">{name}</th>{cells}<td>{fmt(totals(i))}</td></tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
