//! `<SlopeChart>`: each series' change between two points in time on one shared scale.
//! Geometry comes from `hydroxy_charts::rank`.

use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, rank};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour};
use crate::rank_kit::{Dim, SeriesLegend, series_style};

pub use hydroxy_charts::line::Series;

fn pair(s: &Series) -> (f64, f64) {
    (s.values.first().copied().unwrap_or(f64::NAN), s.values.get(1).copied().unwrap_or(f64::NAN))
}

/// Two columns of values joined by a line per series. Each series needs two values: before and after.
/// Hover or touch a line or end to read it; focus the chart and use the arrow keys
/// (Left/Right for the end, Up/Down for the series), Home and End to jump, Escape to clear.
/// Hovering a legend item picks its series out.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SlopeChart(
    /// One series per line; `values` is `[from, to]`.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// Heading over the left axis, such as "2023".
    #[prop(into)]
    from_label: String,
    /// Heading over the right axis, such as "2024".
    #[prop(into)]
    to_label: String,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Formats values in the labels, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| series.with(|s| rank::slope(&s.iter().map(pair).collect::<Vec<_>>())));
    // (series, end): end 0 is "from", 1 is "to".
    let active = RwSignal::new(None::<(usize, usize)>);
    let dim = Dim::new(active);
    let (from_t, to_t) = (from_label.clone(), to_label.clone());
    let describe = move |(si, _): (usize, usize)| {
        series.with(|s| {
            s.get(si).map(|s| {
                let (a, b) = pair(s);
                format!("{}: {from_t} {}, {to_t} {}", s.name, fmt(a), fmt(b))
            })
        })
    };
    let (from_h, to_h) = (from_label.clone(), to_label.clone());
    let (from_tip, to_tip) = (from_label.clone(), to_label.clone());
    let group_label = format!("{label}. Use the arrow keys to step through the lines.");

    view! {
        <figure data-scope="slope-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), series.with(Vec::len), 2) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    let items = series.get();
                    let l = layout.get();
                    let lines = items
                        .iter()
                        .zip(l.lines.iter())
                        .enumerate()
                        .map(|(i, (s, line))| {
                            let (a, b) = pair(s);
                            let dir = if b > a { "up" } else if b < a { "down" } else { "flat" };
                            view! {
                                <g
                                    data-part="series"
                                    data-index=i
                                    data-direction=dir
                                    data-dim=move || dim.of(i)
                                    style=series_style(i)
                                    on:pointerenter=move |_| {
                                        if active.get_untracked().map(|(s, _)| s) != Some(i) {
                                            active.set(Some((i, 1)));
                                        }
                                    }
                                >
                                    <line data-part="line" x1=l.x.0 y1=line.y.0 x2=l.x.1 y2=line.y.1></line>
                                    <circle data-part="point" data-active=move || (active.get() == Some((i, 0))).then_some("") cx=l.x.0 cy=line.y.0 r="4.5"></circle>
                                    <circle data-part="point" data-active=move || (active.get() == Some((i, 1))).then_some("") cx=l.x.1 cy=line.y.1 r="4.5"></circle>
                                    <circle data-part="hit" cx=l.x.0 cy=line.y.0 r="14" on:pointerenter=move |_| active.set(Some((i, 0))) on:pointerdown=move |_| active.set(Some((i, 0)))></circle>
                                    <circle data-part="hit" cx=l.x.1 cy=line.y.1 r="14" on:pointerenter=move |_| active.set(Some((i, 1))) on:pointerdown=move |_| active.set(Some((i, 1)))></circle>
                                    <text data-part="name" x=l.x.0 - 12.0 y=line.label_y.0 text-anchor="end">{format!("{}  {}", s.name, fmt(a))}</text>
                                    <text data-part="name" x=l.x.1 + 12.0 y=line.label_y.1 text-anchor="start">{format!("{}  {}", fmt(b), s.name)}</text>
                                </g>
                            }
                        })
                        .collect_view();
                    view! {
                        <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) aria-hidden="true">
                            <text data-part="column-label" x=l.x.0 y="14" text-anchor="middle">{from_h.clone()}</text>
                            <text data-part="column-label" x=l.x.1 y="14" text-anchor="middle">{to_h.clone()}</text>
                            {lines}
                        </svg>
                    }
                }}
                {move || {
                    let (si, end) = active.get()?;
                    let item = series.with(|s| s.get(si).cloned())?;
                    let l = layout.get();
                    let line = l.lines.get(si)?;
                    let (x, y) = if end == 0 { (l.x.0, line.y.0) } else { (l.x.1, line.y.1) };
                    let (from, to) = pair(&item);
                    let change = to - from;
                    let sign = if change > 0.0 { "+" } else { "" };
                    let tip = Tip {
                        x: x / l.width,
                        y: 1.0 - y / l.height,
                        title: item.name,
                        rows: vec![
                            TipRow::new(None, from_tip.clone(), fmt(from)),
                            TipRow::new(None, to_tip.clone(), fmt(to)),
                            TipRow::new(None, "Change", format!("{sign}{}", fmt(change))),
                        ],
                    };
                    let side = if end == 0 { "right" } else { "left" };
                    Some(view! { <ChartTip tip swatch=colour(si) anchored=true side /> })
                }}
            </div>
            <SeriesLegend series dim />
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(&describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr><th scope="col">"Series"</th><th scope="col">{from_label}</th><th scope="col">{to_label}</th></tr>
                </thead>
                <tbody>
                    {move || {
                        series
                            .get()
                            .into_iter()
                            .map(|s| {
                                let (a, b) = pair(&s);
                                view! { <tr><th scope="row">{s.name}</th><td>{fmt(a)}</td><td>{fmt(b)}</td></tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
