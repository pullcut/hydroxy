//! `<SpiralChart>`: a time series wound onto a spiral, one turn per period, each value a
//! short arc shaded by value. Geometry comes from `hydroxy_charts::spiral`.

use hydroxy_charts::{format_value, raster, spiral};
use leptos::prelude::*;

use crate::chart_frame::provide_export;
use crate::line_chart::plot_fractions;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// A spiral that turns once per `period` values, so the same moment in every period lines
/// up along a spoke and seasonal patterns show as wedges of colour. The spiral draws out
/// from the centre. Hover or touch an arc to read it; focus the chart and use Left/Right to
/// step along the spiral, Up/Down to move a whole turn, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn SpiralChart(
    /// The values in time order. Non-finite values leave a gap.
    #[prop(into)]
    values: Signal<Vec<f64>>,
    /// A name for each value, such as its date, for the tooltip and table.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// Values per turn: 365 for daily values over years, 24 × 7 for hourly values over weeks.
    period: f64,
    /// Names spaced evenly around the dial, clockwise from 12 o'clock, such as the months.
    #[prop(optional, into)]
    spokes: Signal<Vec<String>>,
    /// A name for each turn, from the centre out, such as the years. Shown at 12 o'clock.
    #[prop(optional, into)]
    turns: Signal<Vec<String>>,
    /// What the chart shows, such as "Daily cycling distance". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot diameter in px.
    #[prop(default = 360)]
    size: u16,
    /// Formats values for the tooltip, legend and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| values.with(|v| spiral::layout(v, period)));
    let active = RwSignal::new(None::<usize>);
    let len = move || layout.with(|l| l.segments.len());
    let name = move |i: usize| labels.with(|l| l.get(i).cloned().unwrap_or_else(|| (i + 1).to_string()));
    let value_text = move |v: Option<f64>| v.map_or_else(|| "No data".to_string(), fmt);
    let hover = move |ev: leptos::ev::PointerEvent| {
        let Some((x, y)) = plot_fractions(&ev) else { return };
        let next = layout.with(|l| spiral::index_at(l, period, x, 1.0 - y));
        if active.get_untracked() != next {
            active.set(next);
        }
    };
    provide_export(move || {
        let l = layout.get_untracked();
        std::iter::once(vec![String::new(), "Value".to_string()])
            .chain(l.segments.iter().enumerate().map(|(i, s)| vec![name(i), s.value.map_or_else(String::new, fmt)]))
            .collect()
    });
    let describe =
        move |i: usize| layout.with(|l| l.segments.get(i).map(|s| format!("{}: {}", name(i), value_text(s.value))));

    view! {
        <figure data-scope="spiral-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the left and right arrow keys to step along the spiral, up and down to move a turn.")
                style=format!("width: {size}px;")
                on:pointermove=hover
                on:pointerdown=hover
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = spiral::key(&ev.key(), active.get_untracked(), len(), period) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox="0 0 100 100" aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        let count = spokes.with(Vec::len);
                        (0..count).map(|j| view! { <path data-part="spoke" d=spiral::spoke(&l, j, count).0></path> }).collect_view()
                    }}
                    {move || {
                        let l = layout.get();
                        let width = format!("{:.3}", l.spacing * 0.78);
                        l.segments
                            .into_iter()
                            .map(|s| {
                                let style = format!("stroke: {}; --t0: {:.4}; --dt: {:.4};", spiral::shade(s.level), s.start, s.span);
                                view! {
                                    <path
                                        data-part="segment"
                                        data-empty=s.value.is_none().then_some("")
                                        pathLength="1"
                                        stroke-width=width.clone()
                                        style=style
                                        d=s.path
                                    ></path>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let i = active.get()?;
                        let (d, w) = layout.with(|l| l.segments.get(i).map(|s| (s.path.clone(), format!("{:.3}", l.spacing * 0.78))))?;
                        Some(view! { <path data-part="highlight" stroke-width=w d=d></path> })
                    }}
                </svg>
                {move || {
                    let l = layout.get();
                    spokes
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(j, text)| {
                            let (_, (x, y)) = spiral::spoke(&l, j, spokes.with(Vec::len));
                            view! {
                                <span data-part="spoke-label" data-side=spiral::side(x) aria-hidden="true" style=format!("left: {}; top: {};", pct(x), pct(y))>
                                    {text}
                                </span>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let l = layout.get();
                    turns
                        .get()
                        .into_iter()
                        .take(l.turns)
                        .enumerate()
                        .map(|(k, text)| {
                            let (x, y) = spiral::turn_anchor(&l, k);
                            view! { <span data-part="turn-label" aria-hidden="true" style=format!("left: {}; top: {};", pct(x), pct(y))>{text}</span> }
                        })
                        .collect_view()
                }}
                {move || {
                    let i = active.get()?;
                    let (x, y) = layout.with(|l| spiral::anchor(l, period, i));
                    let value = layout.with(|l| l.segments.get(i).and_then(|s| s.value));
                    Some(view! {
                        <div data-part="tooltip" data-side={if x > 0.5 { "left" } else { "right" }} style=format!("left: {}; top: {};", pct(x), pct(y)) aria-hidden="true">
                            <div data-part="tooltip-label">{name(i)}</div>
                            <div data-part="tooltip-value">{value_text(value)}</div>
                        </div>
                    })
                }}
            </div>
            <div data-part="scale" aria-hidden="true">
                <span data-part="scale-value">{move || fmt(layout.with(|l| l.min))}</span>
                <span data-part="ramp"></span>
                <span data-part="scale-value">{move || fmt(layout.with(|l| l.max))}</span>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>
                    {label.clone()}
                    {move || {
                        let total = len();
                        let shown = raster::table_rows(total, 2);
                        (shown < total).then(|| format!(" (first {shown} of {total} values)"))
                    }}
                </caption>
                <thead>
                    <tr>
                        <th scope="col"></th>
                        <th scope="col">"Value"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        let shown = raster::table_rows(l.segments.len(), 2);
                        l.segments
                            .iter()
                            .take(shown)
                            .enumerate()
                            .map(|(i, s)| view! { <tr><th scope="row">{name(i)}</th><td>{s.value.map_or_else(|| "–".to_string(), fmt)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
