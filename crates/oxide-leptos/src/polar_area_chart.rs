//! `<PolarAreaChart>`: a Nightingale rose. Every category gets the same angle and its
//! radius encodes the value. Geometry comes from `oxide_charts::polar`.

use leptos::prelude::*;
use oxide_charts::{format_value, funnel, nav, polar};

pub use oxide_charts::Datum;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Which side of its anchor a label around the rose sits on.
fn side(x: f64) -> &'static str {
    if x < 0.45 {
        "left"
    } else if x > 0.55 {
        "right"
    } else {
        "center"
    }
}

/// A rose of equal-angle wedges, clockwise from 12 o'clock. Hover, touch or focus a
/// wedge to read it; focus the chart and use the arrow keys to step through wedges,
/// Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn PolarAreaChart(
    /// The categories, clockwise from 12 o'clock. Months and hours read well.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Incidents by month". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Rose diameter in px.
    #[prop(default = 280)]
    size: u16,
    /// Formats values for the rings, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| data.with(|d| polar::layout(&d.iter().map(|x| x.value).collect::<Vec<_>>(), 0.03)));
    let active = RwSignal::new(None::<usize>);
    let len = move || data.with(Vec::len);
    let describe = move |i: usize| data.with(|d| d.get(i).map(|x| format!("{}: {}", x.label, fmt(x.value))));

    view! {
        <figure data-scope="polar-area-chart" data-part="root" style=move || format!("--ox-segments: {};", len())>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to step through categories.")
                style=format!("width: {size}px;")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let key = ev.key();
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active.get_untracked(), len()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox="0 0 100 100" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .rings
                            .into_iter()
                            .map(|r| view! { <circle data-part="ring" cx="50" cy="50" r=format!("{:.3}", r.radius * polar::R)></circle> })
                            .collect_view()
                    }}
                    {move || {
                        layout
                            .get()
                            .wedges
                            .into_iter()
                            .enumerate()
                            .map(|(i, w)| {
                                view! {
                                    <path
                                        data-part="wedge"
                                        data-index=i
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {i};")
                                        d=w.path
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                    ></path>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    layout
                        .get()
                        .rings
                        .into_iter()
                        .map(|r| {
                            view! {
                                <span data-part="ring-label" aria-hidden="true" style=format!("top: {};", pct(0.5 - r.radius / 2.0))>
                                    {fmt(r.value)}
                                </span>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let l = layout.get();
                    data.get()
                        .into_iter()
                        .zip(l.wedges)
                        .enumerate()
                        .map(|(i, (d, w))| {
                            let (x, y) = polar::at(1.06, w.mid);
                            view! {
                                <span
                                    data-part="category"
                                    data-index=i
                                    data-side=side(x)
                                    aria-hidden="true"
                                    style=format!("left: {}; top: {};", pct(x), pct(y))
                                >
                                    {d.label}
                                </span>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let i = active.get()?;
                    let datum = data.with(|d| d.get(i).cloned())?;
                    let wedge = layout.with(|l| l.wedges.get(i).cloned())?;
                    let (x, y) = polar::at(wedge.radius.max(0.2) * 0.66, wedge.mid);
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-side={if x > 0.5 { "left" } else { "right" }}
                                style=format!("left: {}; top: {};", pct(x), pct(y))
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style="background: var(--chart-1);"></span>
                                    {datum.label}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Value"</span>
                                        <span data-part="tooltip-value">{fmt(datum.value)}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Category"</th>
                        <th scope="col">"Value"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        data.get()
                            .into_iter()
                            .map(|d| {
                                view! {
                                    <tr>
                                        <th scope="row">{d.label}</th>
                                        <td>{fmt(d.value)}</td>
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
