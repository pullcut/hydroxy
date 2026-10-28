//! `<RadarChart>`: several series over N axes. Geometry comes from `hydroxy_charts::radar`.

use hydroxy_charts::{format_value, nav, radar};
use leptos::prelude::*;

pub use hydroxy_charts::line::Series;

/// Series compared across the same axes. Axes start at zero and share one scale.
/// Hover or touch a vertex to read it; focus the chart and use the arrow keys
/// (Left/Right around the axes, Up/Down between series), Home and End to jump, Escape
/// to clear. Hovering a legend item picks its series out.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
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
    /// Formats values on the rings, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let value_at = move |s: &Series, i: usize| {
        s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let layout = Memo::new(move |_| {
        let n = axes.with(Vec::len);
        series.with(|s| radar::layout(n, &s.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), rings, max))
    });
    // (series, axis)
    let active = RwSignal::new(None::<(usize, usize)>);
    let legend_hover = RwSignal::new(None::<usize>);
    let focus = move || active.get().map(|(s, _)| s).or_else(|| legend_hover.get());
    let dim = move |i: usize| focus().is_some_and(|f| f != i).then_some("");
    let axis_name = move |k: usize| axes.with(|a| a.get(k).cloned().unwrap_or_default());
    let describe = move |(si, k): (usize, usize)| {
        series.with(|s| s.get(si).map(|s| format!("{}, {}: {}", s.name, axis_name(k), value_at(s, k))))
    };
    let group_label = format!("{label}. Use the arrow keys to step through the values.");

    view! {
        <figure data-scope="radar-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let (rows, cols) = (series.with(Vec::len), axes.with(Vec::len));
                    if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), rows, cols) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    let names = axes.get();
                    let l = layout.get();
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
                    let shapes = l
                        .shapes
                        .iter()
                        .enumerate()
                        .map(|(i, shape)| {
                            let dots = shape
                                .points
                                .iter()
                                .enumerate()
                                .map(|(k, &(x, y))| {
                                    view! {
                                        <circle data-part="point" data-active=move || (active.get() == Some((i, k))).then_some("") cx=x cy=y r="3.5"></circle>
                                        <circle
                                            data-part="hit"
                                            cx=x
                                            cy=y
                                            r="12"
                                            on:pointerenter=move |_| active.set(Some((i, k)))
                                            on:pointerdown=move |_| active.set(Some((i, k)))
                                        ></circle>
                                    }
                                })
                                .collect_view();
                            view! {
                                <g data-part="series" data-index=i data-dim=move || dim(i) style=format!("--c: var(--chart-{});", i % 6 + 1)>
                                    <path data-part="area" d=shape.path.clone()></path>
                                    {dots}
                                </g>
                            }
                        })
                        .collect_view();
                    view! {
                        <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) aria-hidden="true">
                            {web}
                            {spokes}
                            {shapes}
                        </svg>
                    }
                }}
                {move || {
                    let (si, k) = active.get()?;
                    let l = layout.get();
                    let (x, y) = *l.shapes.get(si)?.points.get(k)?;
                    let item = series.with(|s| s.get(si).cloned())?;
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if x < l.centre.0 { "left" } else { "right" }}
                                style=format!("left: {:.3}%; top: {:.3}%;", x / l.width * 100.0, y / l.height * 100.0)
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: var(--chart-{});", si % 6 + 1)></span>
                                    {item.name.clone()}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">{axis_name(k)}</span>
                                        <span data-part="tooltip-value">{value_at(&item, k)}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <Show when=move || series.with(|s| s.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        series
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, s)| {
                                view! {
                                    <li
                                        data-part="legend-item"
                                        data-index=i
                                        data-dim=move || dim(i)
                                        on:pointerenter=move |_| legend_hover.set(Some(i))
                                        on:pointerleave=move |_| legend_hover.set(None)
                                    >
                                        <span data-part="swatch" style=format!("background: var(--chart-{});", i % 6 + 1)></span>
                                        {s.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Axis"</th>
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
