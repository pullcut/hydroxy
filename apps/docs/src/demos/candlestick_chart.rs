use hydroxy_leptos::candlestick_chart::{Annotation, Candle, CandlestickChart};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    // A 30-day run of daily prices with volume, from a deterministic wobble.
    let mut price = 142.0;
    let mut candles = Vec::new();
    let mut labels = Vec::new();
    for i in 0..30 {
        let t = f64::from(i);
        let open = price;
        let close = open + (t * 0.9).sin() * 4.0 + (t * 2.3).cos() * 2.0 + 0.4;
        let high = open.max(close) + 1.0 + f64::from(i % 4);
        let low = open.min(close) - 1.0 - f64::from(i % 3);
        let volume = 800_000.0 + f64::from((i * 37) % 11) * 90_000.0;
        candles.push(Candle::new(open, high, low, close).with_volume(volume));
        labels.push(format!("{} Sep", i + 1));
        price = close;
    }

    // A resistance level and the earnings week, by candle index.
    let levels = vec![Annotation::y_line(150.0, "Resistance"), Annotation::x_band(9.0, 13.0, "Earnings")];

    view! {
        // Hover, or focus the plot and press Left/Right, to read one day.
        // Scroll or pinch to zoom, drag to pan, or press + and − on the focused plot.
        <CandlestickChart
            candles=candles
            labels=labels
            label="ACME daily prices"
            format=|v: f64| format!("${v:.2}")
            annotations=levels
            zoom=true
        />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "candlestick-chart",
    name: "Candlestick chart",
    group: "Finance",
    summary: "Show open, high, low and close per period. A candle that closed at or above its open is green and one that closed lower is red. Candles with a volume add a volume strip underneath.",
    demo,
    leptos: include_str!("candlestick_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/candlestick_chart.rs"),
    parts: &[
        &["CandlestickChart", "root", "figure with a screen-reader table of every candle"],
        &["", "columns", "Focusable plot that drives the readout"],
        &["", "wick", "Thin line from the low to the high"],
        &["", "body", "Open to close; colour comes from the direction"],
        &["", "tooltip", "Open, high, low, close and volume of the active candle"],
        &["", "volume", "Optional strip of volume bars, tinted like their candles"],
        &["", "crosshair", "Dashed rule at the active candle's close, with its price in the gutter"],
        &["", "annotation", "Reference line, band or labelled point; x is a candle index"],
        &["", "zoom-reset", "\"Reset zoom\" button, shown while zoomed"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move to the previous or next candle"],
        &["Home / End", "Jump to the first or last candle"],
        &["Escape", "Hide the readout"],
        &["+ / -", "With zoom: zoom in or out around the middle"],
        &["Shift + ArrowLeft / ArrowRight", "With zoom: pan"],
        &["0", "With zoom: reset"],
    ],
    props: &[
        &["candles", "Vec<Candle>", "Required. Candle::new(open, high, low, close), optionally .with_volume(v)"],
        &["labels", "Vec<String>", "Required. One label per candle"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Price plot height in px. Default 260"],
        &["volume", "bool", "Show the volume strip when candles have volumes. Default true"],
        &["format", "Fn(f64) -> String", "Formats prices"],
        &[
            "annotations",
            "Vec<Annotation>",
            "Reference lines, bands and labelled points; x is a candle index, y a price",
        ],
        &["scale", "ScaleKind", "Linear (default), Log or Symlog price axis"],
        &[
            "zoom",
            "bool",
            "Wheel or pinch to zoom, drag to pan, + / - and Shift+arrows on the focused plot. The price axis refits",
        ],
    ],
};
