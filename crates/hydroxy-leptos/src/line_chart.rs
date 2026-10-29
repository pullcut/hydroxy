//! `<LineChart>` and `<AreaChart>`: series over a shared x axis, with a crosshair
//! tooltip. Geometry comes from `hydroxy_charts::line`; the SVG stretches to fill its
//! container and everything else is positioned with CSS percentages.

use hydroxy_charts::annotate;
use hydroxy_charts::brush::Brush;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::zoom::{Axes, Gesture, View};
use hydroxy_charts::{format_value, line, raster, scale, small_multiples};
use leptos::prelude::*;

use crate::chart_frame::provide_export;
use crate::plot_kit::{colour, pct};

mod area_chart;
mod brush_strip;
mod gestures;

pub use area_chart::AreaChart;
use brush_strip::BrushStrip;
pub(crate) use gestures::{
    annotation_views, cut, gesture_down, gesture_move, gutter, link_view, plot_fractions, wheel_zoom,
};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::{Band, Curve, Series};
pub use hydroxy_charts::scale::ScaleKind;

/// Lines over a shared x axis. Hover, touch or focus the plot and use Left/Right to
/// read every series at one x position; Escape hides the crosshair.
#[allow(clippy::fn_params_excessive_bools, reason = "independent on/off props")]
#[component]
pub fn LineChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table. Leave empty with `x`
    /// set to label points from `x` (dates with `time_axis`), which saves building a string
    /// per point for large data.
    #[prop(optional, into)]
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
    /// Reference lines, bands and labelled points. `x` values are point indices, or
    /// numeric x positions when `x` is set. The value axis grows to fit them.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog value axis.
    #[prop(optional)]
    scale: ScaleKind,
    /// Treat `x` as Unix seconds and label the axis with calendar ticks instead of `labels`.
    #[prop(optional)]
    time_axis: bool,
    /// Wheel or pinch to zoom around the pointer, drag to pan, and `+`/`-`/Shift+arrows on the
    /// focused plot. Works with `brush`: both move the same window.
    #[prop(optional)]
    zoom: bool,
    /// A value range the axis must cover, such as the shared domain of small multiples.
    #[prop(optional, into)]
    y_domain: MaybeProp<(f64, f64)>,
    /// The visible window, to control it from outside (link charts by sharing one signal).
    /// Only the x span is used. Default: the chart keeps its own.
    #[prop(optional, into)]
    view: MaybeProp<View>,
    /// Called once a zoom, pan, brush drag or reset has settled, with the new view.
    #[prop(optional, into)]
    on_view_change: Option<Callback<View>>,
    /// Called when the point under the pointer or keyboard changes (the first series' value at
    /// that position), and with `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on the active position.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    // The plot draws a zoomed slice of the full data; everything below uses the slice.
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    // one String per point; a lazy label lookup would save memory past ~10M points.
    let full_labels = Signal::derive(move || {
        let given = labels.get();
        if !given.is_empty() {
            return given;
        }
        x.with(|x| {
            x.as_ref()
                .map(|x| x.iter().map(|&v| if time_axis { scale::date_time(v) } else { fmt(v) }).collect())
                .unwrap_or_default()
        })
    });
    let full_labels = Memo::new(move |_| full_labels.get());
    let full_labels = Signal::from(full_labels);
    let (full_series, full_x, full_bands) = (series, x, bands);
    let window = RwSignal::new(Brush::FULL);
    let full_xs = Memo::new(move |_| {
        let len = full_labels.with(Vec::len);
        full_x.with(|x| line::x_positions(len, x.as_deref()))
    });
    let range = Memo::new(move |_| {
        if brush || zoom { full_xs.with(|xs| window.get().range(xs)) } else { 0..full_labels.with(Vec::len) }
    });
    // Once zoomed, annotations stop replaying their entrance as the view moves.
    let settled = RwSignal::new(false);
    // Wheel and drag steps land at most once per frame; `view` sees one still waiting.
    let pending = StoredValue::new(None::<View>);
    let current_view = move || {
        pending.get_value().unwrap_or_else(|| {
            let b = window.get_untracked();
            View::x_only((b.start, b.end))
        })
    };
    let set_view = move |v: View| {
        settled.set(true);
        if pending.get_value().is_none() {
            request_animation_frame(move || {
                if let Some(v) = pending.try_update_value(Option::take).flatten() {
                    window.set(Brush { start: v.x.0, end: v.x.1 });
                }
            });
        }
        pending.set_value(Some(v));
    };
    link_view(
        move || view.get().map(|v| View::x_only(v.x)),
        on_view_change,
        move || {
            let b = window.get();
            View::x_only((b.start, b.end))
        },
        move |v| {
            settled.set(true);
            window.set(Brush { start: v.x.0, end: v.x.1 });
        },
    );
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let min_span = move || 1.0 / full_labels.with(Vec::len).saturating_sub(1).max(1) as f64;
    let gesture = StoredValue::new(Gesture::default());
    // Memos, so a hover never re-cuts the data.
    let series = Memo::new(move |_| {
        let r = range.get();
        full_series.with(|s| s.iter().map(|s| Series::new(s.name.clone(), cut(&s.values, &r))).collect::<Vec<_>>())
    });
    let labels = Memo::new(move |_| full_labels.with(|l| cut(l, &range.get())));
    let x = Memo::new(move |_| {
        let r = range.get();
        full_x.with(|x| x.as_ref().map(|x| cut(x, &r)))
    });
    let bands = Memo::new(move |_| {
        let r = range.get();
        full_bands
            .with(|b| b.iter().map(|b| Band::new(b.series, cut(&b.low, &r), cut(&b.high, &r))).collect::<Vec<_>>())
    });
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
            let extent = small_multiples::widen(annotations.with(|a| annotate::y_extent(a)), y_domain.get());
            bands.with(|b| line::layout_scaled(&values, b, &xs, curve, 5, scale, extent))
        })
    });
    let placed = Memo::new(move |_| {
        let (len, start) = (labels.with(Vec::len), range.get().start);
        #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
        let offset = if x.with(Option::is_some) { 0.0 } else { start as f64 };
        let y = layout.with(|l| l.scale);
        x.with(|x| {
            annotations
                .with(|a| annotate::place(a, |at| line::x_fraction(at - offset, len, x.as_deref()), |v| y.map(v)))
        })
    });
    let time_labels = Memo::new(move |_| x.with(|x| line::time_labels(x.as_deref(), time_axis)));
    let active = RwSignal::new(None::<usize>);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()? + range.with(|r| r.start);
            full_labels.with(|l| full_series.with(|s| EventDatum::from_series(s, 0, i, l.get(i).map(String::as_str))))
        })
    });
    crate::callbacks::watch_hover(datum, on_hover);
    let value_at = move |s: &Series, i: usize| {
        s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let move_to = move |ev: leptos::ev::PointerEvent| {
        if zoom && gesture.with_value(Gesture::is_active) {
            if let Some(next) = gesture_move(gesture, &ev, current_view(), Axes::X, min_span()) {
                active.set(None);
                set_view(next);
            }
            return;
        }
        let Some((fx, _)) = plot_fractions(&ev) else { return };
        let i = layout.with(|l| line::nearest_index(&l.xs, fx));
        if active.get_untracked() != i {
            active.set(i);
        }
    };
    let press = move |ev: leptos::ev::PointerEvent| {
        if zoom {
            gesture_down(gesture, &ev);
        }
        move_to(ev);
    };
    let release = move |ev: leptos::ev::PointerEvent| gesture.update_value(|g| g.up(ev.pointer_id()));
    provide_export(move || {
        full_labels.with(|l| full_series.with(|s| full_bands.with(|b| line::export_table(l, s, b, &fmt))))
    });
    let x_at = move |i: usize| layout.with(|l| l.xs.get(i).copied().unwrap_or(0.0));
    let announcement = move || {
        active.get().map_or_else(String::new, |i| labels.with(|l| series.with(|s| line::announce(l, s, i, &fmt))))
    };

    view! {
        <figure
            data-scope="line-chart"
            data-part="root"
            data-area=area.then_some("")
            data-scale=scale.attr()
            data-zoom=zoom.then_some("")
            data-settled=move || settled.get().then_some("")
            style=move || gutter(&layout.with(|l| l.gridlines.iter().map(|g| fmt(g.value)).collect::<Vec<_>>()), 36.0)
        >
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        layout
                            .with(|l| l.gridlines.clone())
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
                        let drawn = layout.with(|l| l.bands.clone());
                        drawn
                            .into_iter()
                            .zip(bands.get())
                            .map(|(d, b)| view! { <path data-part="band" data-index=b.series d=d fill=colour(b.series)></path> })
                            .collect_view()
                    }}
                    {move || {
                        let paths: Vec<(String, String)> = layout.with(|l| l.series.iter().map(|s| (s.line.clone(), s.area.clone())).collect());
                        paths
                            .into_iter()
                            .enumerate()
                            .map(|(i, (line_d, area_d))| {
                                let c = colour(i);
                                view! {
                                    <g data-part="series" data-index=i style=format!("--i: {i};")>
                                        {area.then(|| view! { <path data-part="area" d=area_d fill=c.clone()></path> })}
                                        <path data-part="line" d=line_d stroke=c pathLength="1"></path>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                <div data-part="annotations" aria-hidden="true">{move || annotation_views(placed.get())}</div>
                <div
                    data-part="overlay"
                    tabindex="0"
                    role="group"
                    aria-label=if zoom {
                        format!("{label}. Use the left and right arrow keys to read values, plus and minus to zoom, Shift and the arrow keys to pan.")
                    } else {
                        hydroxy_charts::nav::hinted(&label, hydroxy_charts::nav::HINT_READ_VALUES_LR)
                    }
                    on:pointermove=move_to
                    on:pointerdown=press
                    on:pointerup=release
                    on:pointercancel=release
                    on:pointerleave=move |_| active.set(None)
                    on:wheel=move |ev| {
                        if let Some(next) = zoom.then(|| wheel_zoom(&ev, current_view(), Axes::X, min_span())).flatten() {
                            set_view(next);
                        }
                    }
                    on:blur=move |_| active.set(None)
                    on:click=move |_| crate::callbacks::select_active(datum, on_select)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            crate::callbacks::select_active(datum, on_select);
                        }
                        if let Some(next) = zoom.then(|| current_view().key(&ev.key(), ev.shift_key(), Axes::X, min_span())).flatten() {
                            ev.prevent_default();
                            set_view(next);
                            return;
                        }
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
                                let heights: Vec<(usize, f64)> = layout
                                    .with(|l| l.series.iter().enumerate().filter_map(|(si, s)| Some((si, s.points.get(i).copied().flatten()?))).collect());
                                let dots = heights
                                    .into_iter()
                                    .map(|(si, f)| {
                                        view! {
                                            <span
                                                data-part="dot"
                                                style=format!("left: {}; bottom: {}; background: {};", pct(x), pct(f), colour(si))
                                            ></span>
                                        }
                                    })
                                    .collect_view();
                                let cells: Vec<(String, String)> = series.with(|s| s.iter().map(|s| (s.name.clone(), value_at(s, i))).collect());
                                let rows = cells
                                    .into_iter()
                                    .enumerate()
                                    .map(|(si, (name, value))| {
                                        view! {
                                            <li data-part="tooltip-row">
                                                <span data-part="swatch" style=format!("background: {};", colour(si))></span>
                                                <span data-part="tooltip-name">{name}</span>
                                                <span data-part="tooltip-value">{value}</span>
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
                <Show when=move || zoom && !window.get().is_full()>
                    <button data-part="zoom-reset" type="button" on:click=move |_| window.set(Brush::FULL)>
                        "Reset zoom"
                    </button>
                </Show>
            </div>
            <div data-part="x-labels" aria-hidden="true">
                {move || {
                    let ticks = time_labels.get();
                    if ticks.is_empty() {
                        let shown: Vec<(usize, String)> = labels.with(|l| line::label_indices(l.len(), 6).into_iter().map(|i| (i, l[i].clone())).collect());
                        shown
                            .into_iter()
                            .map(|(i, t)| view! { <span data-part="x-label" style=format!("left: {};", pct(x_at(i)))>{t}</span> })
                            .collect_view()
                    } else {
                        ticks.into_iter().map(|(f, t)| view! { <span data-part="x-label" style=format!("left: {};", pct(f))>{t}</span> }).collect_view()
                    }
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
                <caption>
                    {label}
                    {move || {
                        let n = full_labels.with(Vec::len);
                        (n > raster::TABLE_LIMIT).then(|| format!(" (first {} of {n} points)", raster::TABLE_LIMIT))
                    }}
                </caption>
                <thead>
                    <tr>
                        <td></td>
                        {move || full_series.get().into_iter().map(|s| view! { <th scope="col">{s.name}</th> }).collect_view()}
                        {move || {
                            full_bands
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
                        let s: Vec<Series> = full_series.with(|s| s.iter().map(|s| Series::new(s.name.clone(), cut(&s.values, &(0..raster::TABLE_LIMIT)))).collect());
                        let b = full_bands.get();
                        full_labels
                            .with(|l| cut(l, &(0..raster::TABLE_LIMIT)))
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
