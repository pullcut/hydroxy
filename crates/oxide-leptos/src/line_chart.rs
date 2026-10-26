//! `<LineChart>` and `<AreaChart>`: series over a shared x axis, with a crosshair
//! tooltip. Geometry comes from `oxide_charts::line`; the SVG stretches to fill its
//! container and everything else is positioned with CSS percentages.

use leptos::prelude::*;
use oxide_charts::{format_value, line};

pub use oxide_charts::line::{Curve, Series};

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Lines over a shared x axis. Hover, touch or focus the plot and use Left/Right to
/// read every series at one x position; Escape hides the crosshair.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn LineChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// Numeric x positions (ascending), one per label, for uneven spacing. Default: evenly spaced.
    #[prop(optional, into)]
    x: MaybeProp<Vec<f64>>,
    /// What the chart shows, such as "Weekly active users". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 220)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[prop(optional)]
    curve: Curve,
    /// Fill the area under each line.
    #[prop(optional)]
    area: bool,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        let len = labels.with(Vec::len);
        let xs = x.with(|x| line::x_positions(len, x.as_deref()));
        series.with(|s| {
            let values: Vec<Vec<f64>> = s.iter().map(|s| s.values.clone()).collect();
            line::layout(&values, &xs, curve, 5)
        })
    });
    let active = RwSignal::new(None::<usize>);
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    let move_to = move |ev: leptos::ev::PointerEvent| {
        let Some(el) = ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok()) else {
            return;
        };
        let width = f64::from(el.offset_width().max(1));
        let i = layout.with(|l| line::nearest_index(&l.xs, f64::from(ev.offset_x()) / width));
        if active.get_untracked() != i {
            active.set(i);
        }
    };
    let x_at = move |i: usize| layout.with(|l| l.xs.get(i).copied().unwrap_or(0.0));
    let announcement = move || {
        active.get().map_or_else(String::new, |i| {
            let name = labels.with(|l| l.get(i).cloned().unwrap_or_default());
            let parts: Vec<String> = series.with(|s| s.iter().map(|s| format!("{} {}", s.name, value_at(s, i))).collect());
            format!("{name}: {}", parts.join(", "))
        })
    };

    view! {
        <figure data-scope="line-chart" data-part="root" data-area=area.then_some("")>
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
                                        style=format!("bottom: {};", pct(g.fraction))
                                    >
                                        <span data-part="tick">{fmt(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .series
                            .into_iter()
                            .enumerate()
                            .map(|(i, s)| {
                                let c = colour(i);
                                view! {
                                    <g data-part="series" data-index=i style=format!("--i: {i};")>
                                        {area.then(|| view! { <path data-part="area" d=s.area fill=c.clone()></path> })}
                                        <path data-part="line" d=s.line stroke=c pathLength="1"></path>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                <div
                    data-part="overlay"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the left and right arrow keys to read values.")
                    on:pointermove=move_to
                    on:pointerdown=move_to
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let len = labels.with(Vec::len);
                        if let Some(next) = line::crosshair_key(&ev.key(), active.get_untracked(), len) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        active
                            .get()
                            .map(|i| {
                                let x = x_at(i);
                                let dots = layout
                                    .get()
                                    .series
                                    .into_iter()
                                    .enumerate()
                                    .filter_map(|(si, s)| {
                                        s.points.get(i).copied().flatten().map(|f| {
                                            view! {
                                                <span
                                                    data-part="dot"
                                                    style=format!("left: {}; bottom: {}; background: {};", pct(x), pct(f), colour(si))
                                                ></span>
                                            }
                                        })
                                    })
                                    .collect_view();
                                let rows = series
                                    .get()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(si, s)| {
                                        view! {
                                            <li data-part="tooltip-row">
                                                <span data-part="swatch" style=format!("background: {};", colour(si))></span>
                                                <span data-part="tooltip-name">{s.name.clone()}</span>
                                                <span data-part="tooltip-value">{value_at(&s, i)}</span>
                                            </li>
                                        }
                                    })
                                    .collect_view();
                                view! {
                                    <div data-part="crosshair" style=format!("left: {};", pct(x))></div>
                                    {dots}
                                    <div
                                        data-part="tooltip"
                                        data-side={if x > 0.5 { "left" } else { "right" }}
                                        style=format!("left: {};", pct(x))
                                        aria-hidden="true"
                                    >
                                        <div data-part="tooltip-label">{labels.with(|l| l.get(i).cloned().unwrap_or_default())}</div>
                                        <ul>{rows}</ul>
                                    </div>
                                }
                            })
                    }}
                </div>
            </div>
            <div data-part="x-labels" aria-hidden="true">
                {move || {
                    let l = labels.get();
                    line::label_indices(l.len(), 6)
                        .into_iter()
                        .map(|i| view! { <span data-part="x-label" style=format!("left: {};", pct(x_at(i)))>{l[i].clone()}</span> })
                        .collect_view()
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
                                    <li data-part="legend-item" data-index=i>
                                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                        {s.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{announcement}</span>
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
                        labels
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, l)| {
                                let cells = s.iter().map(|s| view! { <td>{value_at(s, i)}</td> }).collect_view();
                                view! { <tr><th scope="row">{l}</th>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}

/// A [`LineChart`] with the area under each line filled.
#[component]
pub fn AreaChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// Numeric x positions (ascending), one per label, for uneven spacing. Default: evenly spaced.
    #[prop(optional, into)]
    x: MaybeProp<Vec<f64>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 220)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[prop(optional)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    view! { <LineChart series labels x label height curve area=true format=format.unwrap_or_else(|| Callback::new(format_value)) /> }
}
