//! `<LineChart>` and `<AreaChart>`: series over a shared x axis, with a crosshair
//! tooltip. Geometry comes from `oxide_charts::line`; the SVG stretches to fill its
//! container and everything else is positioned with CSS percentages.

use leptos::prelude::*;
use oxide_charts::brush::{Brush, Drag, Grab};
use oxide_charts::{format_value, line};

pub use oxide_charts::line::{Band, Curve, Series};

fn cut<T: Clone>(v: &[T], r: &std::ops::Range<usize>) -> Vec<T> {
    v.get(r.start.min(v.len())..r.end.min(v.len())).unwrap_or_default().to_vec()
}

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
    /// Confidence bands: a shaded low-to-high region around a series, picked by index.
    /// The value axis grows to fit them. Default: none.
    #[prop(optional, into)]
    bands: Signal<Vec<Band>>,
    /// Add a brush strip under the plot. Drag its edges or window, or focus an edge and use
    /// Left/Right (Shift for bigger steps, Home/End for the limits) to zoom the plot to a range.
    #[prop(optional)]
    brush: bool,
) -> impl IntoView {
    // The plot draws a zoomed slice of the full data; everything below uses the slice.
    let (full_series, full_labels, full_x, full_bands) = (series, labels, x, bands);
    let window = RwSignal::new(Brush::FULL);
    let full_xs = Memo::new(move |_| {
        let len = full_labels.with(Vec::len);
        full_x.with(|x| line::x_positions(len, x.as_deref()))
    });
    let range = Memo::new(move |_| if brush { full_xs.with(|xs| window.get().range(xs)) } else { 0..full_labels.with(Vec::len) });
    let series = Signal::derive(move || {
        let r = range.get();
        full_series.with(|s| s.iter().map(|s| Series::new(s.name.clone(), cut(&s.values, &r))).collect::<Vec<_>>())
    });
    let labels = Signal::derive(move || full_labels.with(|l| cut(l, &range.get())));
    let x = Signal::derive(move || {
        let r = range.get();
        full_x.with(|x| x.as_ref().map(|x| cut(x, &r)))
    });
    let bands = Signal::derive(move || {
        let r = range.get();
        full_bands.with(|b| b.iter().map(|b| Band::new(b.series, cut(&b.low, &r), cut(&b.high, &r))).collect::<Vec<_>>())
    });
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let band_cell = move |b: &Band, high: bool, i: usize| {
        let v = if high { &b.high } else { &b.low };
        v.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let band_name = move |b: &Band| series.with(|s| s.get(b.series).map(|s| s.name.clone()).unwrap_or_default());

    let layout = Memo::new(move |_| {
        let len = labels.with(Vec::len);
        let xs = x.with(|x| line::x_positions(len, x.as_deref()));
        series.with(|s| {
            let values: Vec<Vec<f64>> = s.iter().map(|s| s.values.clone()).collect();
            bands.with(|b| line::layout_with_bands(&values, b, &xs, curve, 5))
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
                        let drawn = layout.get().bands;
                        drawn
                            .into_iter()
                            .zip(bands.get())
                            .map(|(d, b)| view! { <path data-part="band" data-index=b.series d=d fill=colour(b.series)></path> })
                            .collect_view()
                    }}
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
            {brush
                .then(|| {
                    let overview = Memo::new(move |_| {
                        let values: Vec<Vec<f64>> = full_series.with(|s| s.iter().map(|s| s.values.clone()).collect());
                        full_xs.with(|xs| line::layout(&values, xs, curve, 3).series.into_iter().map(|s| s.line).collect())
                    });
                    view! { <BrushStrip window overview labels=full_labels range label=label.clone() /> }
                })}
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
                        {move || {
                            bands
                                .get()
                                .iter()
                                .flat_map(|b| {
                                    let name = band_name(b);
                                    [view! { <th scope="col">{format!("{name} low")}</th> }, view! { <th scope="col">{format!("{name} high")}</th> }]
                                })
                                .collect_view()
                        }}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let s = series.get();
                        let b = bands.get();
                        labels
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, l)| {
                                let cells = s.iter().map(|s| view! { <td>{value_at(s, i)}</td> }).collect_view();
                                let band_cells = b
                                    .iter()
                                    .flat_map(|b| [view! { <td>{band_cell(b, false, i)}</td> }, view! { <td>{band_cell(b, true, i)}</td> }])
                                    .collect_view();
                                view! { <tr><th scope="row">{l}</th>{cells}{band_cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}

fn strip_fraction(ev: &leptos::ev::PointerEvent) -> Option<f64> {
    let el = ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok())?;
    Some((f64::from(ev.offset_x()) / f64::from(el.offset_width().max(1))).clamp(0.0, 1.0))
}

/// The overview strip under a zoomable [`LineChart`]: a faint copy of every line over the
/// full range, with a draggable window and two keyboard-operable edges.
#[component]
fn BrushStrip(
    window: RwSignal<Brush>,
    overview: Memo<Vec<String>>,
    labels: Signal<Vec<String>>,
    range: Memo<std::ops::Range<usize>>,
    label: String,
) -> impl IntoView {
    let drag = StoredValue::new(None::<Drag>);
    let len = move || labels.with(Vec::len);
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let step = move || 1.0 / len().saturating_sub(1).max(1) as f64;
    let on_down = move |ev: leptos::ev::PointerEvent| {
        let Some(f) = strip_fraction(&ev) else { return };
        if let Some(el) = ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok()) {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        let (next, d) = window.get_untracked().begin(f, 0.025);
        window.set(next);
        drag.set_value(Some(d));
    };
    let on_move = move |ev: leptos::ev::PointerEvent| {
        if let (Some(d), Some(f)) = (drag.get_value(), strip_fraction(&ev)) {
            window.set(d.apply(f, step()));
        }
    };
    let end_drag = move |_| drag.set_value(None);
    let handle_label = label.clone();
    let handle = move |grab: Grab, name: &'static str| {
        let index = move || if grab == Grab::Start { range.get().start } else { range.get().end.saturating_sub(1) };
        view! {
            <div
                data-part="brush-handle"
                data-edge=name
                role="slider"
                tabindex="0"
                aria-label=format!("{handle_label} range {name}")
                aria-valuemin="0"
                aria-valuemax=move || len().saturating_sub(1).to_string()
                aria-valuenow=move || index().to_string()
                aria-valuetext=move || labels.with(|l| l.get(index()).cloned().unwrap_or_default())
                style=move || {
                    let b = window.get();
                    format!("left: {:.3}%;", if grab == Grab::Start { b.start } else { b.end } * 100.0)
                }
                on:keydown=move |ev| {
                    if let Some(next) = window.get_untracked().key(grab, &ev.key(), ev.shift_key(), step(), step()) {
                        ev.prevent_default();
                        window.set(next);
                    }
                }
            ></div>
        }
    };
    view! {
        <div
            data-part="brush"
            role="group"
            aria-label=format!("{label}: zoom range")
            on:pointerdown=on_down
            on:pointermove=on_move
            on:pointerup=end_drag
            on:pointercancel=end_drag
        >
            <svg data-part="brush-marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {move || {
                    overview
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, d)| view! { <path data-part="brush-line" d=d stroke=colour(i)></path> })
                        .collect_view()
                }}
            </svg>
            <div
                data-part="brush-window"
                style=move || {
                    let b = window.get();
                    format!("left: {:.3}%; width: {:.3}%;", b.start * 100.0, b.span() * 100.0)
                }
            ></div>
            {handle(Grab::Start, "start")}
            {handle(Grab::End, "end")}
        </div>
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
