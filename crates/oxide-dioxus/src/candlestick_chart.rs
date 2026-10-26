//! `CandlestickChart`: open, high, low and close per period, with an optional volume
//! strip. Geometry comes from `oxide_charts::candle`; layout is plain CSS.

use dioxus::prelude::*;
use oxide_charts::{candle, format_value, line};

pub use oxide_charts::candle::Candle;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// OHLC candles: green when the period closed at or above its open, red when below.
/// Hover, or focus the plot and use Left/Right to read one candle; Escape hides it.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn CandlestickChart(
    /// The candles, oldest first. Give them volumes to get the volume strip.
    candles: ReadSignal<Vec<Candle>>,
    /// One label per candle, such as a date. Used on the axis, tooltip and table.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "ACME daily". Used as the accessible name.
    label: String,
    /// Price plot height in px, excluding the volume strip and axis labels.
    #[props(default = 260)]
    height: u16,
    /// Show the volume strip when any candle has a volume.
    #[props(default = true)]
    volume: bool,
    /// Formats prices for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let candles_v = candles();
    let labels_v = labels();
    let len = candles_v.len();
    let layout = candle::layout(&candles_v, 5);
    let vol = |c: &Candle| c.volume.filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), candle::compact);
    let label_at = |i: usize| labels_v.get(i).cloned().unwrap_or_default();
    let current = active().filter(|&i| i < len);
    let announcement = current.map_or_else(String::new, |i| {
        let c = candles_v[i];
        format!("{}: open {}, high {}, low {}, close {}", label_at(i), fmt(c.open), fmt(c.high), fmt(c.low), fmt(c.close))
    });

    rsx! {
        figure { "data-scope": "candlestick-chart", "data-part": "root",
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.gridlines.iter() {
                        div { "data-part": "gridline", style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {fmt(g.value)} }
                        }
                    }
                }
                div {
                    "data-part": "columns",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the left and right arrow keys to read candles.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = line::crosshair_key(&e.key().to_string(), active(), len) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for (i , geom) in layout.candles.iter().enumerate() {
                        div {
                            key: "{i}",
                            "data-part": "column",
                            "data-active": (current == Some(i)).then_some(""),
                            onpointerenter: move |_| active.set(Some(i)),
                            if let Some(g) = geom {
                                div {
                                    "data-part": "candle",
                                    "data-dir": if g.up { "up" } else { "down" },
                                    div { "data-part": "wick", style: "bottom: {pct(g.low)}; height: {pct(g.high - g.low)};" }
                                    div {
                                        "data-part": "body",
                                        style: "bottom: {pct(g.open.min(g.close))}; height: {pct((g.open - g.close).abs())};",
                                    }
                                }
                            }
                            if current == Some(i) {
                                div {
                                    "data-part": "tooltip",
                                    "data-side": if i * 3 < len { "start" } else if i * 3 >= len * 2 { "end" } else { "" },
                                    "aria-hidden": "true",
                                    div { "data-part": "tooltip-label", {label_at(i)} }
                                    dl {
                                        dt { "Open" }
                                        dd { {fmt(candles_v[i].open)} }
                                        dt { "High" }
                                        dd { {fmt(candles_v[i].high)} }
                                        dt { "Low" }
                                        dd { {fmt(candles_v[i].low)} }
                                        dt { "Close" }
                                        dd { {fmt(candles_v[i].close)} }
                                        dt { "Volume" }
                                        dd { {vol(&candles_v[i])} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if volume && layout.has_volume {
                div { "data-part": "volume", style: "height: 56px;", "aria-hidden": "true",
                    span { "data-part": "volume-title", "Vol" }
                    div { "data-part": "volume-bars",
                        for (i , geom) in layout.candles.iter().enumerate() {
                            div { key: "{i}", "data-part": "volume-slot",
                                if let Some(g) = geom {
                                    div {
                                        "data-part": "volume-bar",
                                        "data-dir": if g.up { "up" } else { "down" },
                                        style: "height: {pct(g.volume)};",
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "labels", "aria-hidden": "true",
                for i in line::label_indices(labels_v.len(), 6) {
                    span {
                        key: "{i}",
                        "data-part": "label",
                        style: "left: {pct((i as f64 + 0.5) / labels_v.len() as f64)};",
                        "{labels_v[i]}"
                    }
                }
            }
            ul { "data-part": "legend", "aria-hidden": "true",
                li { "data-part": "legend-item",
                    span { "data-part": "swatch", "data-dir": "up" }
                    "Closed up"
                }
                li { "data-part": "legend-item",
                    span { "data-part": "swatch", "data-dir": "down" }
                    "Closed down"
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
                        th { scope: "col", "Open" }
                        th { scope: "col", "High" }
                        th { scope: "col", "Low" }
                        th { scope: "col", "Close" }
                        th { scope: "col", "Volume" }
                    }
                }
                tbody {
                    for (i , c) in candles_v.iter().enumerate() {
                        tr { key: "{i}",
                            th { scope: "row", {label_at(i)} }
                            td { {fmt(c.open)} }
                            td { {fmt(c.high)} }
                            td { {fmt(c.low)} }
                            td { {fmt(c.close)} }
                            td { {vol(c)} }
                        }
                    }
                }
            }
        }
    }
}
