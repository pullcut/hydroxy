//! `<GaugeChart>` and `<RadialBarChart>`. Pie charts are covered by `DonutChart`.
//! Geometry comes from `oxide_charts::radial`.

use leptos::prelude::*;
use oxide_charts::{format_value, nav, radial};

pub use oxide_charts::Datum;
pub use oxide_charts::radial::Threshold;

fn share(value: f64, top: f64) -> f64 {
    if top > 0.0 { (value / top * 100.0).clamp(0.0, 100.0) } else { 0.0 }
}

/// One value on a three-quarter arc. Thresholds colour the arc by the band the value
/// reaches and are marked on the track. A changed value glides along the arc. Hover,
/// touch or focus the gauge to read the value and its band; Escape clears.
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
    /// Formats the value, the scale ends, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (box_px, arc_px) = (f64::from(size), f64::from(thickness));
    let centre = box_px / 2.0;
    let radius = radial::gauge(min, min, max, arc_px, box_px, &[]).radius;
    let banded = !thresholds.is_empty();
    let bands = thresholds.clone();
    let layout = Memo::new(move |_| radial::gauge(value.get(), min, max, arc_px, box_px, &thresholds));
    let band = Memo::new(move |_| {
        let (lo, hi) = radial::band(value.get(), min, max, &bands);
        format!("{}–{}", fmt(lo), fmt(hi))
    });
    let active = RwSignal::new(false);
    // Scale ends sit just below the arc's two ends, 45 degrees either side of straight down.
    let end_dx = radius * std::f64::consts::FRAC_1_SQRT_2;
    let end_y = centre + end_dx + arc_px / 2.0 + 4.0;
    let caption_text = label.clone();
    let live_label = label.clone();
    let table_caption = label.clone();
    let name_label = label.clone();

    view! {
        <div
            data-scope="gauge-chart"
            data-part="root"
            role="meter"
            tabindex="0"
            aria-label=name_label
            aria-valuemin=min
            aria-valuemax=max
            aria-valuenow=move || value.get()
            aria-valuetext=move || fmt(value.get())
            data-active=move || active.get().then_some("")
            style=format!("width: {size}px; height: {}px;", end_y + 14.0)
            on:pointerenter=move |_| active.set(true)
            on:pointerdown=move |_| active.set(true)
            on:pointerleave=move |_| active.set(false)
            on:focus=move |_| active.set(true)
            on:blur=move |_| active.set(false)
            on:keydown=move |ev| {
                if ev.key() == "Escape" && active.get_untracked() {
                    ev.prevent_default();
                    active.set(false);
                }
            }
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
            <div data-part="center" style=format!("top: {centre}px;") aria-hidden="true">
                <span data-part="center-value">{move || fmt(value.get())}</span>
                {caption.map(|t| view! { <span data-part="center-label">{t}</span> })}
            </div>
            <span data-part="end" aria-hidden="true" style=format!("left: {}px; top: {end_y}px;", centre - end_dx)>{fmt(min)}</span>
            <span data-part="end" aria-hidden="true" style=format!("left: {}px; top: {end_y}px;", centre + end_dx)>{fmt(max)}</span>
            {move || {
                active.get().then(|| {
                    let l = layout.get();
                    let (x, y) = radial::anchor(box_px, radius, radial::gauge_angle(l.fraction));
                    view! {
                        <div
                            data-part="tooltip"
                            data-anchored=""
                            data-side={if x > 0.5 { "right" } else { "left" }}
                            style=format!("left: {:.1}px; top: {:.1}px;", x * box_px, y * box_px)
                            aria-hidden="true"
                        >
                            <div data-part="tooltip-label">
                                <span data-part="swatch" style=format!("background: {};", l.colour.unwrap_or_else(|| "var(--chart-1)".into()))></span>
                                {caption_text.clone()}
                            </div>
                            <ul>
                                <li data-part="tooltip-row">
                                    <span data-part="tooltip-name">"Value"</span>
                                    <span data-part="tooltip-value">{fmt(value.get())}</span>
                                </li>
                                <li data-part="tooltip-row">
                                    <span data-part="tooltip-name">{if banded { "Band" } else { "Scale" }}</span>
                                    <span data-part="tooltip-value">{move || band.get()}</span>
                                </li>
                            </ul>
                        </div>
                    }
                })
            }}
            <span data-part="sr-live" aria-live="polite">
                {move || if active.get() { format!("{live_label}: {}, {}", fmt(value.get()), band.get()) } else { String::new() }}
            </span>
            <table data-part="sr-table">
                <caption>{table_caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Measure"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">{if banded { "Band" } else { "Scale" }}</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <th scope="row">{label}</th>
                        <td>{move || fmt(value.get())}</td>
                        <td>{move || band.get()}</td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}

/// Concentric progress rings, the first outermost, each filled against `max`. Changed
/// values glide. Hover, touch or focus a ring or legend row to read it; focus the rings
/// and use the arrow keys to step through them, Home and End to jump, Escape to clear.
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
    /// Formats values for the legend, tooltip and table.
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
    let active = RwSignal::new(None::<usize>);
    let set_active = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let describe = move |i: usize| {
        data.with(|d| d.get(i).map(|x| format!("{}: {} of {}, {:.0}%", x.label, fmt(x.value), fmt(ceiling.get()), share(x.value, ceiling.get()))))
    };
    let group_label = format!("{label}. Use the arrow keys to step through rings.");

    view! {
        <figure data-scope="radial-bar-chart" data-part="root" data-active=move || active.get().map(|_| "")>
            <div
                data-part="plot"
                style=format!("width: {size}px; height: {size}px;")
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), rings.with(Vec::len)) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg width=size height=size viewBox=format!("0 0 {size} {size}") aria-hidden="true" data-part="rings">
                    {move || {
                        rings
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, r)| {
                                let stroke = format!("stroke-width: {thickness}px;");
                                view! {
                                    <g
                                        data-part="ring"
                                        data-index=i
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {i};")
                                        on:pointerenter=move |_| set_active(i)
                                        on:pointerdown=move |_| set_active(i)
                                    >
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
                {move || {
                    let i = active.get()?;
                    let ring = rings.with(|r| r.get(i).cloned())?;
                    let item = data.with(|d| d.get(i).cloned())?;
                    let top = ceiling.get();
                    let (x, y) = radial::anchor(f64::from(size), ring.radius, ring.fraction * 360.0);
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if x > 0.5 { "right" } else { "left" }}
                                style=format!("left: {:.3}%; top: {:.3}%;", x * 100.0, y * 100.0)
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: var(--chart-{});", i % 6 + 1)></span>
                                    {item.label}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Value"</span>
                                        <span data-part="tooltip-value">{fmt(item.value)}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">{format!("Of {}", fmt(top))}</span>
                                        <span data-part="tooltip-value">{format!("{:.0}%", share(item.value, top))}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    let top = ceiling.get();
                    data.get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, d)| {
                            view! {
                                <li
                                    data-part="legend-item"
                                    data-index=i
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    on:pointerenter=move |_| set_active(i)
                                    on:pointerleave=move |_| active.set(None)
                                >
                                    <span data-part="swatch" style=format!("background: var(--chart-{});", i % 6 + 1)></span>
                                    <span>{d.label}</span>
                                    <span data-part="legend-value">{fmt(d.value)}</span>
                                    <span data-part="legend-share">{format!("{:.0}%", share(d.value, top))}</span>
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
                        <th scope="col">"Name"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Max"</th>
                        <th scope="col">"Share"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let top = ceiling.get();
                        data.get()
                            .into_iter()
                            .map(|d| {
                                view! {
                                    <tr>
                                        <th scope="row">{d.label}</th>
                                        <td>{fmt(d.value)}</td>
                                        <td>{fmt(top)}</td>
                                        <td>{format!("{:.0}%", share(d.value, top))}</td>
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
