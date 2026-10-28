//! `CandlestickChart`: open, high, low and close per period, with an optional volume
//! strip. Geometry comes from `hydroxy_charts::candle`; layout is plain CSS.

use dioxus::prelude::*;
use hydroxy_charts::brush::Brush;
use hydroxy_charts::zoom::{Axes, Gesture, View};
use hydroxy_charts::{annotate, bar, candle, format_value, line};

use crate::line_chart::{annotation_nodes, cut, gesture_down, gesture_move, wheel_zoom};
use crate::plot_kit::pct;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::candle::Candle;
pub use hydroxy_charts::scale::ScaleKind;

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
    /// Reference lines, bands and labelled points. `x` values are candle indices; `y`
    /// values are prices. The price axis grows to fit them.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog price axis.
    #[props(default)]
    scale: ScaleKind,
    /// Wheel or pinch to zoom around the pointer, drag to pan, and `+`/`-`/Shift+arrows on
    /// the focused plot.
    #[props(default)]
    zoom: bool,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let mut window = use_signal(|| Brush::FULL);
    let gesture = use_signal(Gesture::default);
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    // The plot draws a zoomed slice of the candles; the table always has them all.
    let all_candles = candles();
    let all_labels = labels();
    let range = if zoom { window().range(&line::x_positions(all_candles.len(), None)) } else { 0..all_candles.len() };
    #[allow(clippy::cast_precision_loss, reason = "candle counts are small")]
    let min_span = 2.0 / all_candles.len().saturating_sub(1).max(2) as f64;
    let view = move || {
        let b = window();
        View::x_only((b.start, b.end))
    };
    // Once zoomed, candles stop replaying their entrance every time the view moves.
    let mut settled = use_signal(|| false);
    let mut set_view = move |v: View| {
        settled.set(true);
        window.set(Brush { start: v.x.0, end: v.x.1 });
    };
    let candles_v = cut(&all_candles, &range);
    let labels_v = cut(&all_labels, &range);
    let len = candles_v.len();
    let notes = annotations();
    let layout = candle::layout_scaled(&candles_v, 5, scale, annotate::y_extent(&notes));
    #[allow(clippy::cast_precision_loss, reason = "candle counts are small")]
    let start = range.start as f64;
    let placed = annotate::place(&notes, |at| bar::category_centre(at - start, len), |v| layout.scale.map(v));
    let release = move |e: Event<PointerData>| {
        let mut g = gesture;
        g.write().up(e.data().pointer_id());
    };
    let vol = |c: &Candle| c.volume.filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), candle::compact);
    let label_at = |i: usize| labels_v.get(i).cloned().unwrap_or_default();
    let current = active().filter(|&i| i < len);
    let announcement = current.map_or_else(String::new, |i| {
        let c = candles_v[i];
        format!(
            "{}: open {}, high {}, low {}, close {}",
            label_at(i),
            fmt(c.open),
            fmt(c.high),
            fmt(c.low),
            fmt(c.close)
        )
    });

    rsx! {
        figure {
            "data-scope": "candlestick-chart",
            "data-settled": settled().then_some(""),
            "data-part": "root",
            "data-scale": scale.attr(),
            "data-zoom": zoom.then_some(""),
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.gridlines.iter() {
                        div { "data-part": "gridline", style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {fmt(g.value)} }
                        }
                    }
                }
                div { "data-part": "annotations", "aria-hidden": "true", {annotation_nodes(&placed)} }
                div {
                    "data-part": "columns",
                    tabindex: "0",
                    role: "group",
                    "aria-label": if zoom {
                        format!("{label}. Use the left and right arrow keys to read candles, plus and minus to zoom, Shift and the arrow keys to pan.")
                    } else {
                        format!("{label}. Use the left and right arrow keys to read candles.")
                    },
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            size.set((s.width, s.height));
                        }
                    },
                    onpointerdown: move |e| {
                        if zoom {
                            gesture_down(gesture, &e, size());
                        }
                    },
                    onpointermove: move |e| {
                        if zoom
                            && gesture.read().active()
                            && let Some(next) = gesture_move(gesture, &e, size(), view(), Axes::X, min_span)
                        {
                            active.set(None);
                            set_view(next);
                        }
                    },
                    onpointerup: release,
                    onpointercancel: release,
                    onwheel: move |e| {
                        if let Some(next) = zoom.then(|| wheel_zoom(&e, size(), view(), Axes::X, min_span)).flatten() {
                            set_view(next);
                        }
                    },
                    onpointerleave: move |e| {
                        release(e);
                        active.set(None);
                    },
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        let key = e.key().to_string();
                        if let Some(next) = zoom.then(|| view().key(&key, e.modifiers().shift(), Axes::X, min_span)).flatten() {
                            e.prevent_default();
                            set_view(next);
                            return;
                        }
                        if let Some(next) = line::crosshair_key(&key, active(), len) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    if let Some((i , g)) = current.and_then(|i| layout.candles.get(i).copied().flatten().map(|g| (i, g))) {
                        div { "data-part": "crosshair", style: "bottom: {pct(g.close)};",
                            span { "data-part": "crosshair-value", {fmt(candles_v[i].close)} }
                        }
                    }
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
                                    style: "--i: {i.min(60)};",
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
                if zoom && !window().is_full() {
                    button { "data-part": "zoom-reset", r#type: "button", onclick: move |_| window.set(Brush::FULL), "Reset zoom" }
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
                                        style: "height: {pct(g.volume)}; --i: {i.min(60)};",
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
                    for (i , c) in all_candles.iter().enumerate() {
                        tr { key: "{i}",
                            th { scope: "row", {all_labels.get(i).cloned().unwrap_or_default()} }
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
