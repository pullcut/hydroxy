//! `<BulletChart>`: one row per measure, each a bar against a target tick over grey
//! qualitative bands. Each row has its own zero-based scale, from `hydroxy_charts::bullet`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{bullet, format_value, funnel, nav};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::{colour, pct};

pub use hydroxy_charts::bullet::BulletRow;

/// Name of band `j`: the given label, or "Range n".
fn band_name(names: &[String], j: usize) -> String {
    names.get(j).cloned().unwrap_or_else(|| format!("Range {}", j + 1))
}

/// Which band `value` falls in, if any.
fn band_of(edges: &[f64], value: f64) -> Option<usize> {
    edges.iter().position(|&e| value <= e)
}

/// Bullet rows. Hover, touch or focus a row to read it; focus the chart and use the
/// arrow keys to step through rows, Home and End to jump, Escape to clear.
#[component]
pub fn BulletChart(
    /// The measures, top to bottom.
    #[prop(into)]
    rows: Signal<Vec<BulletRow>>,
    /// What the chart shows, such as "Quarterly targets". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names for the bands from the lowest up, such as "Poor", "Fair", "Good".
    #[prop(optional, into)]
    range_labels: Vec<String>,
    /// Formats values for the ticks, tooltip and table.
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
    let layouts = Memo::new(move |_| rows.with(|r| r.iter().map(bullet::layout).collect::<Vec<_>>()));
    let active = RwSignal::new(None::<usize>);
    let len = move || rows.with(Vec::len);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            rows.with(|r| r.get(i).map(|r| EventDatum::category(i, r.label.clone(), r.value)))
        })
    });
    watch_hover(datum, on_hover);
    let names = StoredValue::new(range_labels);
    let describe = move |i: usize| {
        let r = rows.with(|r| r.get(i).cloned())?;
        let mut text = format!("{}: {}", r.label, fmt(r.value));
        if let Some(t) = r.target {
            text = format!("{text}, target {}", fmt(t));
        }
        let band = layouts.with(|l| l.get(i).and_then(|l| band_of(&l.edges, r.value)));
        if let Some(j) = band {
            text = format!("{text}, {}", names.with_value(|n| band_name(n, j)));
        }
        Some(text)
    };
    let bands = move || layouts.with(|l| l.iter().map(|l| l.edges.len()).max().unwrap_or(0));
    let has_target = move || rows.with(|r| r.iter().any(|r| r.target.is_some()));
    let row_view = move |label: &'static str, value: String| {
        view! {
            <li data-part="tooltip-row">
                <span data-part="tooltip-name">{label}</span>
                <span data-part="tooltip-value">{value}</span>
            </li>
        }
    };

    view! {
        <figure data-scope="bullet-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to step through measures.")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let key = ev.key();
                    if key == "Enter" {
                        select_active(datum, on_select);
                    }
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active.get_untracked(), len()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    let n = rows.with(Vec::len);
                    rows.get()
                        .into_iter()
                        .zip(layouts.get())
                        .enumerate()
                        .map(|(i, (r, l))| {
                            let value_text = fmt(r.value);
                            let tip_at = l.value;
                            let band = band_of(&l.edges, r.value);
                            view! {
                                <div
                                    data-part="row"
                                    data-index=i
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    style=format!("--i: {i}; --ox-n: {n};")
                                    on:pointerenter=move |_| active.set(Some(i))
                                    on:pointerdown=move |_| active.set(Some(i))
                                    on:click=move |_| {
                                        active.set(Some(i));
                                        select_active(datum, on_select);
                                    }
                                >
                                    <div data-part="row-label" aria-hidden="true">
                                        <span data-part="row-name">{r.label.clone()}</span>
                                        <span data-part="row-value">{value_text.clone()}</span>
                                    </div>
                                    <div data-part="track" aria-hidden="true">
                                        {l.bands.iter().enumerate().map(|(j, &(a, b))| view! {
                                            <span data-part="band" data-band=j style=format!("left: {}; width: {};", pct(a), pct(b - a))></span>
                                        }).collect_view()}
                                        <span data-part="measure" style=format!("width: {};", pct(l.value))></span>
                                        {l.target.map(|t| view! { <span data-part="target" style=format!("left: {};", pct(t))></span> })}
                                        <div data-part="ticks">
                                            {l.ticks.iter().map(|t| view! {
                                                <span data-part="tick" style=format!("left: {};", pct(t.fraction))>{fmt(t.value)}</span>
                                            }).collect_view()}
                                        </div>
                                        <Show when=move || active.get() == Some(i)>
                                            <div
                                                data-part="tooltip"
                                                data-side={if tip_at > 0.6 { "left" } else { "right" }}
                                                style=format!("left: {}; top: 50%;", pct(tip_at))
                                            >
                                                <div data-part="tooltip-label">
                                                    <span data-part="swatch" style=format!("background: {};", colour(0))></span>
                                                    {r.label.clone()}
                                                </div>
                                                <ul>
                                                    {row_view("Value", value_text.clone())}
                                                    {r.target.map(|t| row_view("Target", fmt(t)))}
                                                    {band.map(|j| row_view("Band", names.with_value(|n| band_name(n, j))))}
                                                </ul>
                                            </div>
                                        </Show>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <ul data-part="legend" aria-hidden="true">
                <li data-part="legend-item">
                    <span data-part="key" data-key="measure"></span>
                    "Value"
                </li>
                <Show when=has_target>
                    <li data-part="legend-item">
                        <span data-part="key" data-key="target"></span>
                        "Target"
                    </li>
                </Show>
                {move || {
                    (0..bands())
                        .map(|j| {
                            view! {
                                <li data-part="legend-item">
                                    <span data-part="key" data-key="band" data-band=j></span>
                                    {names.with_value(|n| band_name(n, j))}
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Measure"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Target"</th>
                        {move || (0..bands()).map(|j| view! { <th scope="col">{names.with_value(|n| band_name(n, j))}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cols = bands();
                        rows.get()
                            .into_iter()
                            .zip(layouts.get())
                            .map(|(r, l)| {
                                view! {
                                    <tr>
                                        <th scope="row">{r.label}</th>
                                        <td>{fmt(r.value)}</td>
                                        <td>{r.target.map(fmt).unwrap_or_default()}</td>
                                        {(0..cols).map(|j| view! { <td>{l.edges.get(j).map(|&e| fmt(e)).unwrap_or_default()}</td> }).collect_view()}
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
