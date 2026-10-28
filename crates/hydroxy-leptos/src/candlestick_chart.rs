//! `<CandlestickChart>`: open, high, low and close per period, with an optional volume
//! strip. Geometry comes from `hydroxy_charts::candle`; layout is plain CSS.

use hydroxy_charts::brush::Brush;
use hydroxy_charts::zoom::{Axes, Gesture, View};
use hydroxy_charts::{annotate, bar, candle, format_value, line};
use leptos::prelude::*;

use crate::line_chart::{annotation_views, cut, gesture_down, gesture_move, wheel_zoom};
use crate::plot_kit::pct;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::candle::Candle;
pub use hydroxy_charts::scale::ScaleKind;

/// OHLC candles: green when the period closed at or above its open, red when below.
/// Hover, or focus the plot and use Left/Right to read one candle; Escape hides it.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn CandlestickChart(
    /// The candles, oldest first. Give them volumes to get the volume strip.
    #[prop(into)]
    candles: Signal<Vec<Candle>>,
    /// One label per candle, such as a date. Used on the axis, tooltip and table.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows, such as "ACME daily". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Price plot height in px, excluding the volume strip and axis labels.
    #[prop(default = 260)]
    height: u16,
    /// Show the volume strip when any candle has a volume.
    #[prop(default = true)]
    volume: bool,
    /// Formats prices for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` values are candle indices; `y`
    /// values are prices. The price axis grows to fit them.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog price axis.
    #[prop(optional)]
    scale: ScaleKind,
    /// Wheel or pinch to zoom around the pointer, drag to pan, and `+`/`-`/Shift+arrows on
    /// the focused plot.
    #[prop(optional)]
    zoom: bool,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    // The plot draws a zoomed slice of the candles; the table always has them all.
    let (full_candles, full_labels) = (candles, labels);
    let window = RwSignal::new(Brush::FULL);
    let range = Memo::new(move |_| {
        let n = full_candles.with(Vec::len);
        if zoom { window.get().range(&line::x_positions(n, None)) } else { 0..n }
    });
    let candles = Signal::derive(move || full_candles.with(|c| cut(c, &range.get())));
    let labels = Signal::derive(move || full_labels.with(|l| cut(l, &range.get())));
    let view = move || {
        let b = window.get_untracked();
        View::x_only((b.start, b.end))
    };
    // Once zoomed, candles stop replaying their entrance every time the view moves.
    let settled = RwSignal::new(false);
    let set_view = move |v: View| {
        settled.set(true);
        window.set(Brush { start: v.x.0, end: v.x.1 });
    };
    #[allow(clippy::cast_precision_loss, reason = "candle counts are small")]
    let min_span = move || 2.0 / full_candles.with(Vec::len).saturating_sub(1).max(2) as f64;
    let gesture = StoredValue::new(Gesture::default());
    let layout = Memo::new(move |_| {
        let extent = annotations.with(|a| annotate::y_extent(a));
        candles.with(|c| candle::layout_scaled(c, 5, scale, extent))
    });
    let placed = Memo::new(move |_| {
        let (n, start) = (candles.with(Vec::len), range.get().start);
        let y = layout.with(|l| l.scale);
        #[allow(clippy::cast_precision_loss, reason = "candle counts are small")]
        annotations.with(|a| annotate::place(a, |at| bar::category_centre(at - start as f64, n), |v| y.map(v)))
    });
    let active = RwSignal::new(None::<usize>);
    let vol = move |c: &Candle| c.volume.filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), candle::compact);
    let label_at = move |i: usize| labels.with(|l| l.get(i).cloned().unwrap_or_default());
    let announcement = move || {
        active.get().and_then(|i| candles.with(|c| c.get(i).copied()).map(|c| (i, c))).map_or_else(
            String::new,
            |(i, c)| {
                format!(
                    "{}: open {}, high {}, low {}, close {}",
                    label_at(i),
                    fmt(c.open),
                    fmt(c.high),
                    fmt(c.low),
                    fmt(c.close)
                )
            },
        )
    };
    let show_volume = move || volume && layout.with(|l| l.has_volume);

    view! {
        <figure
            data-scope="candlestick-chart"
            data-part="root"
            data-scale=scale.attr()
            data-zoom=zoom.then_some("")
            data-settled=move || settled.get().then_some("")
        >
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .gridlines
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div data-part="gridline" style=format!("bottom: {};", pct(g.fraction))>
                                        <span data-part="tick">{fmt(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div data-part="annotations" aria-hidden="true">{move || annotation_views(placed.get())}</div>
                <div
                    data-part="columns"
                    tabindex="0"
                    role="group"
                    aria-label=if zoom {
                        format!("{label}. Use the left and right arrow keys to read candles, plus and minus to zoom, Shift and the arrow keys to pan.")
                    } else {
                        format!("{label}. Use the left and right arrow keys to read candles.")
                    }
                    on:pointerdown=move |ev| {
                        if zoom {
                            gesture_down(gesture, &ev);
                        }
                    }
                    on:pointermove=move |ev| {
                        if let Some(next) = (zoom && gesture.with_value(Gesture::active))
                            .then(|| gesture_move(gesture, &ev, view(), Axes::X, min_span()))
                            .flatten()
                        {
                            active.set(None);
                            set_view(next);
                        }
                    }
                    on:pointerup=move |ev| gesture.update_value(|g| g.up(ev.pointer_id()))
                    on:pointercancel=move |ev| gesture.update_value(|g| g.up(ev.pointer_id()))
                    on:wheel=move |ev| {
                        if let Some(next) = zoom.then(|| wheel_zoom(&ev, view(), Axes::X, min_span())).flatten() {
                            set_view(next);
                        }
                    }
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = zoom.then(|| view().key(&ev.key(), ev.shift_key(), Axes::X, min_span())).flatten() {
                            ev.prevent_default();
                            set_view(next);
                            return;
                        }
                        let len = candles.with(Vec::len);
                        if let Some(next) = line::crosshair_key(&ev.key(), active.get_untracked(), len) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let i = active.get()?;
                        let g = layout.with(|l| l.candles.get(i).copied().flatten())?;
                        let close = candles.with(|c| c.get(i).map(|c| c.close))?;
                        Some(
                            view! {
                                <div data-part="crosshair" style=format!("bottom: {};", pct(g.close))>
                                    <span data-part="crosshair-value">{fmt(close)}</span>
                                </div>
                            },
                        )
                    }}
                    {move || {
                        let n = candles.with(Vec::len);
                        layout
                            .get()
                            .candles
                            .into_iter()
                            .enumerate()
                            .map(|(i, geom)| {
                                let side = if i * 3 < n { "start" } else if i * 3 >= n * 2 { "end" } else { "" };
                                let tip = move || {
                                    (active.get() == Some(i))
                                        .then(|| candles.with(|c| c.get(i).copied()))
                                        .flatten()
                                        .map(|c| {
                                            view! {
                                                <div data-part="tooltip" data-side=side aria-hidden="true">
                                                    <div data-part="tooltip-label">{label_at(i)}</div>
                                                    <dl>
                                                        <dt>"Open"</dt><dd>{fmt(c.open)}</dd>
                                                        <dt>"High"</dt><dd>{fmt(c.high)}</dd>
                                                        <dt>"Low"</dt><dd>{fmt(c.low)}</dd>
                                                        <dt>"Close"</dt><dd>{fmt(c.close)}</dd>
                                                        <dt>"Volume"</dt><dd>{vol(&c)}</dd>
                                                    </dl>
                                                </div>
                                            }
                                        })
                                };
                                view! {
                                    <div
                                        data-part="column"
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| active.set(Some(i))
                                    >
                                        {geom
                                            .map(|g| {
                                                let (lo, hi) = (g.open.min(g.close), g.open.max(g.close));
                                                view! {
                                                    <div data-part="candle" data-dir=if g.up { "up" } else { "down" } style=format!("--i: {};", i.min(60))>
                                                        <div data-part="wick" style=format!("bottom: {}; height: {};", pct(g.low), pct(g.high - g.low))></div>
                                                        <div data-part="body" style=format!("bottom: {}; height: {};", pct(lo), pct(hi - lo))></div>
                                                    </div>
                                                }
                                            })}
                                        {tip}
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <Show when=move || zoom && !window.get().is_full()>
                    <button data-part="zoom-reset" type="button" on:click=move |_| window.set(Brush::FULL)>
                        "Reset zoom"
                    </button>
                </Show>
            </div>
            <Show when=show_volume>
                <div data-part="volume" style="height: 56px;" aria-hidden="true">
                    <span data-part="volume-title">"Vol"</span>
                    <div data-part="volume-bars">
                        {move || {
                            layout
                                .get()
                                .candles
                                .into_iter()
                                .enumerate()
                                .map(|(i, g)| {
                                    view! {
                                        <div data-part="volume-slot">
                                            {g.map(|g| {
                                                view! {
                                                    <div
                                                        data-part="volume-bar"
                                                        data-dir=if g.up { "up" } else { "down" }
                                                        style=format!("height: {}; --i: {};", pct(g.volume), i.min(60))
                                                    ></div>
                                                }
                                            })}
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
            </Show>
            <div data-part="labels" aria-hidden="true">
                {move || {
                    let l = labels.get();
                    line::label_indices(l.len(), 6)
                        .into_iter()
                        .map(|i| {
                            #[allow(clippy::cast_precision_loss, reason = "candle counts are small")]
                            let at = (i as f64 + 0.5) / l.len() as f64;
                            view! { <span data-part="label" style=format!("left: {};", pct(at))>{l[i].clone()}</span> }
                        })
                        .collect_view()
                }}
            </div>
            <ul data-part="legend" aria-hidden="true">
                <li data-part="legend-item"><span data-part="swatch" data-dir="up"></span>"Closed up"</li>
                <li data-part="legend-item"><span data-part="swatch" data-dir="down"></span>"Closed down"</li>
            </ul>
            <span data-part="sr-live" aria-live="polite">{announcement}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <td></td>
                        <th scope="col">"Open"</th>
                        <th scope="col">"High"</th>
                        <th scope="col">"Low"</th>
                        <th scope="col">"Close"</th>
                        <th scope="col">"Volume"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let names = full_labels.get();
                        full_candles
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, c)| {
                                view! {
                                    <tr>
                                        <th scope="row">{names.get(i).cloned().unwrap_or_default()}</th>
                                        <td>{fmt(c.open)}</td>
                                        <td>{fmt(c.high)}</td>
                                        <td>{fmt(c.low)}</td>
                                        <td>{fmt(c.close)}</td>
                                        <td>{vol(&c)}</td>
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
