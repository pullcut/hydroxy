//! `<SlopeChart>`: each series' change between two points in time on one shared scale.
//! Geometry comes from `oxide_charts::rank`. Every line is focusable.

use leptos::prelude::*;
use oxide_charts::{format_value, rank};

pub use oxide_charts::line::Series;

/// Two columns of values joined by a line per series. Each series needs two values: before and after.
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
    /// Formats values in the labels and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let pair = |s: &Series| (s.values.first().copied().unwrap_or(f64::NAN), s.values.get(1).copied().unwrap_or(f64::NAN));
    let (from_t, to_t) = (from_label.clone(), to_label.clone());
    let (from_h, to_h) = (from_label.clone(), to_label.clone());
    let aria = label.clone();
    view! {
        <figure data-scope="slope-chart" data-part="root">
            {move || {
                let items = series.get();
                let l = rank::slope(&items.iter().map(pair).collect::<Vec<_>>());
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
                                style=format!("--c: var(--chart-{});", i % 6 + 1)
                                tabindex="0"
                                role="img"
                                aria-label=format!("{}: {} {} to {} {}", s.name, from_t, fmt(a), to_t, fmt(b))
                            >
                                <line data-part="line" x1=l.x.0 y1=line.y.0 x2=l.x.1 y2=line.y.1></line>
                                <circle data-part="point" cx=l.x.0 cy=line.y.0 r="4.5"></circle>
                                <circle data-part="point" cx=l.x.1 cy=line.y.1 r="4.5"></circle>
                                <text data-part="name" x=l.x.0 - 12.0 y=line.label_y.0 text-anchor="end">{format!("{}  {}", s.name, fmt(a))}</text>
                                <text data-part="name" x=l.x.1 + 12.0 y=line.label_y.1 text-anchor="start">{format!("{}  {}", fmt(b), s.name)}</text>
                            </g>
                        }
                    })
                    .collect_view();
                view! {
                    <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) role="group" aria-label=aria.clone()>
                        <text data-part="column-label" x=l.x.0 y="14" text-anchor="middle">{from_h.clone()}</text>
                        <text data-part="column-label" x=l.x.1 y="14" text-anchor="middle">{to_h.clone()}</text>
                        {lines}
                    </svg>
                }
            }}
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr><td></td><th scope="col">{from_label}</th><th scope="col">{to_label}</th></tr>
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
