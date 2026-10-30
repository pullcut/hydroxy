//! `<ClockChart>`: activity by hour of the day as radial bars around a 24-hour clock
//! face, midnight at the top. Geometry comes from `hydroxy_charts::clock`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot;
use hydroxy_charts::{clock, format_value, funnel, polar};
use leptos::prelude::*;

pub use hydroxy_charts::line::Series;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::{colour, pct};

/// "09:00–10:00" for slot `i` of `n` around the day.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "slot counts are tiny"
)]
fn slot_name(i: usize, n: usize) -> String {
    let minutes = |k: usize| (k as f64 * 1440.0 / n.max(1) as f64).round() as usize;
    let (a, b) = (minutes(i), minutes(i + 1));
    format!("{:02}:{:02}–{:02}:{:02}", a / 60, a % 60, b / 60 % 24, b % 60)
}

/// A 24-hour clock face with a bar per hour growing out from the centre, so the shape of a
/// day (a morning peak, a quiet night) reads at a glance. A second series sits beside the
/// first in each hour, for comparing two days or two groups. Hover or touch an hour to
/// read it; focus the chart and use the arrow keys to step through the hours, Home and
/// End to jump, Escape to clear.
#[component]
pub fn ClockChart(
    /// One or two series with a value per hour from midnight (24 values; other counts split the day evenly).
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// What the chart shows, such as "Support chats by hour". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Face diameter in px.
    #[prop(default = 300)]
    size: u16,
    /// Formats values for the rings, tooltip and table.
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
    let series = Memo::new(move |_| series.with(|s| s.iter().take(2).cloned().collect::<Vec<_>>()));
    let layout =
        Memo::new(move |_| series.with(|s| clock::layout(&s.iter().map(|s| s.values.as_slice()).collect::<Vec<_>>())));
    let active = RwSignal::new(None::<usize>);
    let slots = move || layout.with(|l| l.slots);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            let name = slot_name(i, slots());
            series.with(|s| EventDatum::from_series(s, 0, i, Some(&name)))
        })
    });
    watch_hover(datum, on_hover);
    let describe = move |i: usize| {
        let values = series.with(|s| {
            s.iter().map(|s| format!("{} {}", s.name, plot::cell(s.values.get(i).copied(), &fmt))).collect::<Vec<_>>()
        });
        format!("{}: {}", slot_name(i, slots()), values.join(", "))
    };

    view! {
        <figure data-scope="clock-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to step through the hours.")
                style=format!("width: {size}px;")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        select_active(datum, on_select);
                    }
                    let key = ev.key();
                    if let Some(next) = list_key(funnel::arrow_alias(&key), active.get_untracked(), slots()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox="0 0 100 100" aria-hidden="true">
                    <circle data-part="face" cx="50" cy="50" r=format!("{:.3}", clock::R)></circle>
                    {move || {
                        layout
                            .get()
                            .rings
                            .into_iter()
                            .map(|r| view! { <circle data-part="ring" cx="50" cy="50" r=format!("{:.3}", r.radius)></circle> })
                            .collect_view()
                    }}
                    <circle data-part="baseline" cx="50" cy="50" r=format!("{:.3}", clock::INNER)></circle>
                    {move || {
                        let n = slots();
                        (0..n)
                            .map(|h| {
                                #[allow(clippy::cast_precision_loss, reason = "slot counts are tiny")]
                                let angle = h as f64 / n as f64 * std::f64::consts::TAU;
                                let major = n == 24 && h % 6 == 0;
                                let (x0, y0) = polar::at(if major { 0.9 } else { 0.94 }, angle);
                                let (x1, y1) = polar::at(1.0, angle);
                                view! {
                                    <line
                                        data-part="hour-tick"
                                        data-major=major.then_some("")
                                        x1=format!("{:.3}", x0 * 100.0)
                                        y1=format!("{:.3}", y0 * 100.0)
                                        x2=format!("{:.3}", x1 * 100.0)
                                        y2=format!("{:.3}", y1 * 100.0)
                                    ></line>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let l = layout.get();
                        let width = format!("{:.3}", l.width);
                        let slots = l.slots;
                        l.bars
                            .into_iter()
                            .map(|b| {
                                let slot = b.slot;
                                view! {
                                    <path
                                        data-part="bar"
                                        data-index=b.series
                                        data-empty=b.value.is_none().then_some("")
                                        data-active=move || (active.get() == Some(slot)).then_some("")
                                        pathLength="1"
                                        stroke=colour(b.series)
                                        stroke-width=width.clone()
                                        style=format!("--i: {slot}; --ox-n: {slots};")
                                        d=b.path
                                    ></path>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        layout
                            .get()
                            .hits
                            .into_iter()
                            .enumerate()
                            .map(|(i, d)| {
                                view! {
                                    <path data-part="hit" d=d on:pointerenter=move |_| active.set(Some(i)) on:pointerdown=move |_| active.set(Some(i)) on:click=move |_| { active.set(Some(i)); select_active(datum, on_select); }></path>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let n = slots();
                    let step = if n == 24 { 3 } else { n.div_ceil(8).max(1) };
                    (0..n)
                        .step_by(step)
                        .map(|h| {
                            #[allow(clippy::cast_precision_loss, reason = "slot counts are tiny")]
                            let (x, y) = polar::at(1.1, h as f64 / n as f64 * std::f64::consts::TAU);
                            let text = slot_name(h, n)[..2].to_string();
                            view! { <span data-part="hour" aria-hidden="true" style=format!("left: {}; top: {};", pct(x), pct(y))>{text}</span> }
                        })
                        .collect_view()
                }}
                {move || {
                    layout
                        .get()
                        .rings
                        .into_iter()
                        .map(|r| {
                            view! {
                                <span data-part="ring-label" aria-hidden="true" style=format!("top: {};", pct(0.5 - r.radius / 100.0))>
                                    {fmt(r.value)}
                                </span>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let i = active.get()?;
                    let n = slots();
                    let (x, y) = clock::anchor(i, n);
                    let rows = series.with(|s| {
                        s.iter()
                            .enumerate()
                            .map(|(k, s)| {
                                view! {
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">
                                            <span data-part="swatch" style=format!("background: {};", colour(k))></span>
                                            {s.name.clone()}
                                        </span>
                                        <span data-part="tooltip-value">{plot::cell(s.values.get(i).copied(), &fmt)}</span>
                                    </li>
                                }
                            })
                            .collect_view()
                    });
                    Some(view! {
                        <div data-part="tooltip" data-side={if x > 0.5 { "left" } else { "right" }} style=format!("left: {}; top: {};", pct(x), pct(y)) aria-hidden="true">
                            <div data-part="tooltip-label">{slot_name(i, n)}</div>
                            <ul>{rows}</ul>
                        </div>
                    })
                }}
            </div>
            <Show when=move || series.with(|s| s.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        series
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(k, s)| view! { <li data-part="legend-item"><span data-part="swatch" style=format!("background: {};", colour(k))></span>{s.name}</li> })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Hour"</th>
                        {move || series.get().into_iter().map(|s| view! { <th scope="col">{s.name}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let n = slots();
                        let all = series.get();
                        (0..n)
                            .map(|i| {
                                let cells = all.iter().map(|s| view! { <td>{plot::cell(s.values.get(i).copied(), &fmt)}</td> }).collect_view();
                                view! { <tr><th scope="row">{slot_name(i, n)}</th>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
