//! `<GaugeChart>` (arc, segmented and needle styles) and `<RadialBarChart>`. Pie charts
//! are covered by `DonutChart`.
//! Geometry comes from `hydroxy_charts::radial`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, radial};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::radial::{GaugeVariant, Threshold};

fn share(value: f64, top: f64) -> f64 {
    if top > 0.0 { (value / top * 100.0).clamp(0.0, 100.0) } else { 0.0 }
}

/// One value on a three-quarter arc, in one of three styles (`variant`): a thermometer
/// fill along the arc (the default), the track split into status bands with a pointer, or
/// a needle that sweeps to the value and settles. Thresholds colour the bands. A changed
/// value glides. Hover, touch or focus the gauge to read the value and its band; Escape clears.
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
    /// How the value is drawn: `Arc` (default), `Segmented` or `Needle`.
    #[prop(optional)]
    variant: GaugeVariant,
    /// Called with the gauge's value when it is hovered or focused, and `None` when that ends.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the gauge's value when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (box_px, arc_px) = (f64::from(size), f64::from(thickness));
    let centre = box_px / 2.0;
    let radius = radial::gauge(min, min, max, arc_px, box_px, &[]).radius;
    let banded = !thresholds.is_empty();
    let bands = thresholds.clone();
    // Segmented bands sit apart by a small gap; a needle's thin scale is continuous.
    let gap = if variant == GaugeVariant::Segmented { 2.5 } else { 0.0 };
    let segments = radial::segments(min, max, radius, box_px, &thresholds, gap);
    let lit_segments = segments.clone();
    let lit = Memo::new(move |_| radial::segment_of(&lit_segments, value.get()));
    let layout = Memo::new(move |_| radial::gauge(value.get(), min, max, arc_px, box_px, &thresholds));
    let angle = Memo::new(move |_| layout.with(|l| radial::gauge_angle(l.fraction)));
    let band = Memo::new(move |_| {
        let (lo, hi) = radial::band(value.get(), min, max, &bands);
        format!("{}–{}", fmt(lo), fmt(hi))
    });
    let active = RwSignal::new(false);
    let datum_label = label.clone();
    let datum = Signal::derive(move || active.get().then(|| EventDatum::category(0, datum_label.clone(), value.get())));
    watch_hover(Some(datum), on_hover);
    // Scale ends sit just below the arc's two ends, 45 degrees either side of straight down.
    let end_dx = radius * std::f64::consts::FRAC_1_SQRT_2;
    let end_y = centre + end_dx + arc_px / 2.0 + 4.0;
    // A needle's hub takes the centre, so its figure sits below it, between the arc ends.
    let figure_y = if variant == GaugeVariant::Needle { centre + radius * 0.45 } else { centre };
    let turn = move || format!("--to: {:.2}deg; transform-origin: {centre}px {centre}px;", angle.get());
    let thin = (arc_px * 0.45).max(4.0);
    let caption_text = label.clone();
    let live_label = label.clone();
    let table_caption = label.clone();
    let name_label = label.clone();

    view! {
        <div
            data-scope="gauge-chart"
            data-part="root"
            data-variant=variant.attr()
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
            on:click=move |_| select_active(Some(datum), on_select)
            on:pointerleave=move |_| active.set(false)
            on:focus=move |_| active.set(true)
            on:blur=move |_| active.set(false)
            on:keydown=move |ev| {
                if ev.key() == "Enter" {
                    select_active(Some(datum), on_select);
                }
                if ev.key() == "Escape" && active.get_untracked() {
                    ev.prevent_default();
                    active.set(false);
                }
            }
        >
            <svg width=size height=size viewBox=format!("0 0 {size} {size}") aria-hidden="true">
                {match variant {
                    GaugeVariant::Segmented => {
                        let tip = centre - radius - arc_px / 2.0 - 3.0;
                        view! {
                            {segments
                                .clone()
                                .into_iter()
                                .enumerate()
                                .map(|(i, seg)| {
                                    view! {
                                        <path
                                            data-part="segment"
                                            data-index=i
                                            data-active=move || (lit.get() == Some(i)).then_some("")
                                            d=seg.path
                                            pathLength="1"
                                            style=format!("stroke-width: {thickness}px; stroke: {}; --i: {i};", seg.colour)
                                        ></path>
                                    }
                                })
                                .collect_view()}
                            <g data-part="pointer" style=turn>
                                <path d=format!("M{:.2} {:.2}L{:.2} {:.2}L{centre:.2} {tip:.2}Z", centre - 6.0, tip - 9.0, centre + 6.0, tip - 9.0)></path>
                            </g>
                        }
                            .into_any()
                    }
                    GaugeVariant::Needle => {
                        let scale = if banded {
                            segments
                                .clone()
                                .into_iter()
                                .map(|seg| view! { <path data-part="segment" d=seg.path style=format!("stroke-width: {thin}px; stroke: {};", seg.colour)></path> })
                                .collect_view()
                                .into_any()
                        } else {
                            view! { <path data-part="track" d=move || layout.with(|l| l.track.clone()) style=format!("stroke-width: {thin}px;")></path> }.into_any()
                        };
                        view! {
                            {scale}
                            <g data-part="needle" style=turn>
                                <path
                                    data-part="needle-arm"
                                    d=format!("M{:.2} {centre:.2}L{centre:.2} {:.2}L{:.2} {centre:.2}Z", centre - 3.0, centre - radius + 2.0, centre + 3.0)
                                ></path>
                                <circle data-part="hub" cx=centre cy=centre r=7></circle>
                            </g>
                        }
                            .into_any()
                    }
                    GaugeVariant::Arc => {
                        view! {
                            <path data-part="track" d=move || layout.with(|l| l.track.clone()) style=format!("stroke-width: {thickness}px;")></path>
                            <path
                                data-part="value-arc"
                                d=move || layout.with(|l| l.track.clone())
                                pathLength="1"
                                style=move || {
                                    let l = layout.get();
                                    format!(
                                        "stroke-width: {thickness}px; stroke: {}; stroke-dasharray: {:.4} 1; {}",
                                        l.colour.unwrap_or_else(|| colour(0)),
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
                        }
                            .into_any()
                    }
                }}
            </svg>
            <div data-part="center" style=format!("top: {figure_y}px;") aria-hidden="true">
                <span data-part="center-value">{move || fmt(value.get())}</span>
                {caption.map(|t| view! { <span data-part="center-label">{t}</span> })}
            </div>
            <span data-part="end" aria-hidden="true" style=format!("left: {}px; top: {end_y}px;", centre - end_dx)>{fmt(min)}</span>
            <span data-part="end" aria-hidden="true" style=format!("left: {}px; top: {end_y}px;", centre + end_dx)>{fmt(max)}</span>
            {move || {
                active.get().then(|| {
                    let l = layout.get();
                    let (x, y) = radial::anchor(box_px, radius, radial::gauge_angle(l.fraction));
                    let tip = Tip {
                        x,
                        y: 1.0 - y,
                        title: caption_text.clone(),
                        rows: vec![
                            TipRow::new(None, "Value", fmt(value.get())),
                            TipRow::new(None, if banded { "Band" } else { "Scale" }, band.get()),
                        ],
                    };
                    let swatch = l.colour.unwrap_or_else(|| colour(0));
                    let style = format!("left: {:.1}px; top: {:.1}px;", x * box_px, y * box_px);
                    let tip_side = if x > 0.5 { "right" } else { "left" };
                    view! { <ChartTip tip swatch anchored=true style side=tip_side /> }
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

/// Concentric progress rings, the first outermost, each filled against `max`. Below a full
/// turn (`sweep`), each ring is an arc whose label sits at its start, in the clear quarter
/// left of 12 o'clock. Arcs sweep in one after another; changed values glide. Hover, touch or focus a ring or legend row to read it; focus the rings
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
    /// How far a full ring goes, in degrees clockwise from 12 o'clock. At 270 or less the
    /// labels sit at the arc starts. Default: a full turn.
    #[prop(default = 360.0)]
    sweep: f64,
    /// Called with the ring under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the ring when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let labelled = sweep <= 270.0;
    let centre = f64::from(size) / 2.0;
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let ceiling = Memo::new(move |_| {
        max.get()
            .unwrap_or_else(|| data.with(|d| d.iter().map(|x| x.value).filter(|v| v.is_finite()).fold(0.0, f64::max)))
    });
    let rings = Memo::new(move |_| {
        let values: Vec<f64> = data.with(|d| d.iter().map(|x| x.value).collect());
        radial::arc_rings(&values, ceiling.get(), f64::from(size), f64::from(thickness), 6.0, sweep)
    });
    let active = RwSignal::new(None::<usize>);
    let datum = Signal::derive(move || {
        let i = active.get()?;
        data.with(|d| d.get(i).map(|x| EventDatum::category(i, x.label.clone(), x.value)))
    });
    watch_hover(Some(datum), on_hover);
    let set_active = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let describe = move |i: usize| {
        data.with(|d| {
            d.get(i).map(|x| {
                format!(
                    "{}: {} of {}, {:.0}%",
                    x.label,
                    fmt(x.value),
                    fmt(ceiling.get()),
                    share(x.value, ceiling.get())
                )
            })
        })
    };
    let group_label = format!("{label}. Use the arrow keys to step through rings.");

    view! {
        <figure data-scope="radial-bar-chart" data-part="root" data-labelled=labelled.then_some("") data-active=move || active.get().map(|_| "")>
            <div
                data-part="plot"
                style=format!("width: {size}px; height: {size}px;")
                tabindex="0"
                role="group"
                aria-label=group_label
                on:click=move |_| select_active(Some(datum), on_select)
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        select_active(Some(datum), on_select);
                    }
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
                                let name = labelled.then(|| data.with(|d| d.get(i).map(|x| x.label.clone()))).flatten();
                                let top = centre - r.radius;
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
                                                "{stroke} stroke: {}; stroke-dasharray: {:.4} 1;{}",
                                                colour(i),
                                                r.fraction,
                                                if r.fraction <= 0.0 { " opacity: 0;" } else { "" },
                                            )
                                        ></path>
                                        {name.map(|n| view! {
                                            <text data-part="ring-label" x=centre - 8.0 y=top dy="0.35em" text-anchor="end">{n}</text>
                                        })}
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
                    let (x, y) = radial::anchor(f64::from(size), ring.radius, ring.fraction * sweep.clamp(1.0, 360.0));
                    let tip = Tip {
                        x,
                        y: 1.0 - y,
                        title: item.label,
                        rows: vec![
                            TipRow::new(None, "Value", fmt(item.value)),
                            TipRow::new(None, format!("Of {}", fmt(top)), format!("{:.0}%", share(item.value, top))),
                        ],
                    };
                    let tip_side = if x > 0.5 { "right" } else { "left" };
                    Some(view! { <ChartTip tip swatch=colour(i) anchored=true side=tip_side /> })
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
                                    <span data-part="swatch" style=format!("background: {};", colour(i))></span>
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
