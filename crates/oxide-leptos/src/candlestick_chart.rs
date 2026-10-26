//! `<CandlestickChart>`: open, high, low and close per period, with an optional volume
//! strip. Geometry comes from `oxide_charts::candle`; layout is plain CSS.

use leptos::prelude::*;
use oxide_charts::{candle, format_value, line};

pub use oxide_charts::candle::Candle;

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
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| candles.with(|c| candle::layout(c, 5)));
    let active = RwSignal::new(None::<usize>);
    let pct = |f: f64| format!("{:.3}%", f * 100.0);
    let vol = move |c: &Candle| c.volume.filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), candle::compact);
    let label_at = move |i: usize| labels.with(|l| l.get(i).cloned().unwrap_or_default());
    let announcement = move || {
        active.get().and_then(|i| candles.with(|c| c.get(i).copied()).map(|c| (i, c))).map_or_else(String::new, |(i, c)| {
            format!("{}: open {}, high {}, low {}, close {}", label_at(i), fmt(c.open), fmt(c.high), fmt(c.low), fmt(c.close))
        })
    };
    let show_volume = move || volume && layout.with(|l| l.has_volume);

    view! {
        <figure data-scope="candlestick-chart" data-part="root">
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
                <div
                    data-part="columns"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the left and right arrow keys to read candles.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let len = candles.with(Vec::len);
                        if let Some(next) = line::crosshair_key(&ev.key(), active.get_untracked(), len) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
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
                                                    <div data-part="candle" data-dir=if g.up { "up" } else { "down" }>
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
                                .map(|g| {
                                    view! {
                                        <div data-part="volume-slot">
                                            {g.map(|g| {
                                                view! {
                                                    <div
                                                        data-part="volume-bar"
                                                        data-dir=if g.up { "up" } else { "down" }
                                                        style=format!("height: {};", pct(g.volume))
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
                        candles
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, c)| {
                                view! {
                                    <tr>
                                        <th scope="row">{label_at(i)}</th>
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
