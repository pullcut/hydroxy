//! `<RadarChart>`: several series over N axes. Geometry comes from `oxide_charts::radar`.
//! Every series is focusable; hovering or focusing one picks it out.

use leptos::prelude::*;
use oxide_charts::{format_value, radar};

pub use oxide_charts::line::Series;

#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
/// Series compared across the same axes. Axes start at zero and share one scale.
#[component]
pub fn RadarChart(
    /// The series. `values` has one entry per axis. Up to four stay legible.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One name per axis, clockwise from the top. Three or more.
    #[prop(into)]
    axes: Signal<Vec<String>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Value at the outer ring. Default: the data maximum rounded up to a nice number.
    #[prop(optional)]
    max: Option<f64>,
    /// Number of guide rings.
    #[prop(default = 4)]
    rings: usize,
    /// Formats values on the rings, tooltips and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    let aria = label.clone();
    view! {
        <figure data-scope="radar-chart" data-part="root">
            {move || {
                let names = axes.get();
                let items = series.get();
                let l = radar::layout(names.len(), &items.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), rings, max);
                let web = l
                    .rings
                    .iter()
                    .map(|(v, d)| {
                        let tick = (l.centre.0 + 4.0, l.centre.1 - 130.0 * v / l.max + 10.0);
                        view! {
                            <path data-part="ring" d=d.clone()></path>
                            <text data-part="tick" x=tick.0 y=tick.1>{fmt(*v)}</text>
                        }
                    })
                    .collect_view();
                let spokes = l
                    .axes
                    .iter()
                    .zip(names.iter())
                    .map(|(a, n)| {
                        view! {
                            <line data-part="spoke" x1=l.centre.0 y1=l.centre.1 x2=a.end.0 y2=a.end.1></line>
                            <text data-part="axis-label" x=a.label.0 y=a.label.1 text-anchor=a.anchor>{n.clone()}</text>
                        }
                    })
                    .collect_view();
                let shapes = items
                    .iter()
                    .zip(l.shapes.iter())
                    .enumerate()
                    .map(|(i, (s, shape))| {
                        let summary = names.iter().enumerate().map(|(k, n)| format!("{n} {}", value_at(s, k))).collect::<Vec<_>>().join(", ");
                        let dots = shape
                            .points
                            .iter()
                            .enumerate()
                            .map(|(k, &(x, y))| {
                                let title = format!("{}, {}: {}", s.name, names.get(k).cloned().unwrap_or_default(), value_at(s, k));
                                view! { <circle data-part="point" cx=x cy=y r="3.5"><title>{title}</title></circle> }
                            })
                            .collect_view();
                        view! {
                            <g
                                data-part="series"
                                data-index=i
                                style=format!("--c: var(--chart-{});", i % 6 + 1)
                                tabindex="0"
                                role="img"
                                aria-label=format!("{}: {summary}", s.name)
                            >
                                <path data-part="area" d=shape.path.clone()></path>
                                {dots}
                            </g>
                        }
                    })
                    .collect_view();
                view! {
                    <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) role="group" aria-label=aria.clone()>
                        {web}
                        {spokes}
                        {shapes}
                    </svg>
                }
            }}
            <Show when=move || series.with(|s| s.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        series
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, s)| {
                                view! {
                                    <li data-part="legend-item" data-index=i>
                                        <span data-part="swatch" style=format!("background: var(--chart-{});", i % 6 + 1)></span>
                                        {s.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <td></td>
                        {move || series.get().into_iter().map(|s| view! { <th scope="col">{s.name}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let s = series.get();
                        axes
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, a)| {
                                let cells = s.iter().map(|s| view! { <td>{value_at(s, i)}</td> }).collect_view();
                                view! { <tr><th scope="row">{a}</th>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
