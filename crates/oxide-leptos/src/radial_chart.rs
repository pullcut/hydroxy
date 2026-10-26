//! `<GaugeChart>` and `<RadialBarChart>`. Pie charts are covered by `DonutChart`.
//! Geometry comes from `oxide_charts::radial`.

use leptos::prelude::*;
use oxide_charts::{format_value, radial};

pub use oxide_charts::Datum;
pub use oxide_charts::radial::Threshold;

/// One value on a three-quarter arc. Thresholds colour the arc by the band the value
/// reaches and are marked on the track.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn GaugeChart(
    /// The current value.
    #[prop(into)]
    value: Signal<f64>,
    /// Lowest value on the scale.
    #[prop(default = 0.0)]
    min: f64,
    /// Highest value on the scale.
    #[prop(default = 100.0)]
    max: f64,
    /// What is measured, such as "Support SLA met". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Colour bands: `Threshold::new(from, colour)`. Default: everything in `--chart-1`.
    #[prop(optional, into)]
    thresholds: Vec<Threshold>,
    /// Text under the centre figure, such as "this week".
    #[prop(optional, into)]
    caption: Option<String>,
    /// Diameter in px.
    #[prop(default = 200)]
    size: u16,
    /// Arc width in px.
    #[prop(default = 16)]
    thickness: u16,
    /// Formats the value and the scale ends. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (box_px, arc_px) = (f64::from(size), f64::from(thickness));
    let centre = box_px / 2.0;
    let radius = radial::gauge(min, min, max, arc_px, box_px, &[]).radius;
    let layout = Memo::new(move |_| radial::gauge(value.get(), min, max, arc_px, box_px, &thresholds));
    // Scale ends sit just below the arc's two ends, 45 degrees either side of straight down.
    let end_dx = radius * std::f64::consts::FRAC_1_SQRT_2;
    let end_y = centre + end_dx + arc_px / 2.0 + 4.0;

    view! {
        <div
            data-scope="gauge-chart"
            data-part="root"
            role="meter"
            aria-label=label
            aria-valuemin=min
            aria-valuemax=max
            aria-valuenow=move || value.get()
            aria-valuetext=move || fmt(value.get())
            style=format!("width: {size}px; height: {}px;", end_y + 14.0)
        >
            <svg width=size height=size viewBox=format!("0 0 {size} {size}") aria-hidden="true">
                <path data-part="track" d=move || layout.with(|l| l.track.clone()) style=format!("stroke-width: {thickness}px;")></path>
                <path
                    data-part="value-arc"
                    d=move || layout.with(|l| l.track.clone())
                    pathLength="1"
                    style=move || {
                        let l = layout.get();
                        format!(
                            "stroke-width: {thickness}px; stroke: {}; stroke-dasharray: {:.4} 1; {}",
                            l.colour.unwrap_or_else(|| "var(--chart-1)".into()),
                            l.fraction,
                            if l.fraction <= 0.0 { "opacity: 0;" } else { "" },
                        )
                    }
                ></path>
                {move || {
                    layout
                        .get()
                        .marks
                        .into_iter()
                        .map(|(x1, y1, x2, y2)| view! { <line data-part="mark" x1=x1 y1=y1 x2=x2 y2=y2></line> })
                        .collect_view()
                }}
            </svg>
            <div data-part="center" style=format!("top: {centre}px;")>
                <span data-part="center-value">{move || fmt(value.get())}</span>
                {caption.map(|t| view! { <span data-part="center-label">{t}</span> })}
            </div>
            <span data-part="end" aria-hidden="true" style=format!("left: {}px; top: {end_y}px;", centre - end_dx)>{fmt(min)}</span>
            <span data-part="end" aria-hidden="true" style=format!("left: {}px; top: {end_y}px;", centre + end_dx)>{fmt(max)}</span>
        </div>
    }
}

/// Concentric progress rings, the first outermost, each filled against `max`.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RadialBarChart(
    /// The rings, in display order and categorical colour order. Up to six read well.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Goal progress". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// What a full ring means. Default: the largest value.
    #[prop(optional, into)]
    max: MaybeProp<f64>,
    /// Diameter in px.
    #[prop(default = 200)]
    size: u16,
    /// Ring width in px.
    #[prop(default = 14)]
    thickness: u16,
    /// Formats values for the legend and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let ceiling = Memo::new(move |_| {
        max.get().unwrap_or_else(|| data.with(|d| d.iter().map(|x| x.value).filter(|v| v.is_finite()).fold(0.0, f64::max)))
    });
    let rings = Memo::new(move |_| {
        let values: Vec<f64> = data.with(|d| d.iter().map(|x| x.value).collect());
        radial::rings(&values, ceiling.get(), f64::from(size), f64::from(thickness), 6.0)
    });

    view! {
        <figure data-scope="radial-bar-chart" data-part="root">
            <svg width=size height=size viewBox=format!("0 0 {size} {size}") aria-hidden="true" data-part="rings">
                {move || {
                    rings
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, r)| {
                            let stroke = format!("stroke-width: {thickness}px;");
                            view! {
                                <g data-part="ring" data-index=i style=format!("--i: {i};")>
                                    <path data-part="track" d=r.path.clone() style=stroke.clone()></path>
                                    <path
                                        data-part="fill"
                                        d=r.path
                                        pathLength="1"
                                        style=format!(
                                            "{stroke} stroke: var(--chart-{}); stroke-dasharray: {:.4} 1;{}",
                                            i % 6 + 1,
                                            r.fraction,
                                            if r.fraction <= 0.0 { " opacity: 0;" } else { "" },
                                        )
                                    ></path>
                                </g>
                            }
                        })
                        .collect_view()
                }}
            </svg>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    let top = ceiling.get();
                    data.get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, d)| {
                            let share = if top > 0.0 { (d.value / top * 100.0).clamp(0.0, 100.0) } else { 0.0 };
                            view! {
                                <li data-part="legend-item" data-index=i>
                                    <span data-part="swatch" style=format!("background: var(--chart-{});", i % 6 + 1)></span>
                                    <span>{d.label}</span>
                                    <span data-part="legend-value">{fmt(d.value)}</span>
                                    <span data-part="legend-share">{format!("{share:.0}%")}</span>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <tbody>
                    {move || {
                        let top = ceiling.get();
                        data.get()
                            .into_iter()
                            .map(|d| view! { <tr><th scope="row">{d.label}</th><td>{fmt(d.value)}</td><td>{format!("of {}", fmt(top))}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
