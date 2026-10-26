use leptos::prelude::*;
use oxide_leptos::candlestick_chart::{Candle, CandlestickChart};

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

    view! {
        // Hover, or focus the plot and press Left/Right, to read one day.
        <CandlestickChart candles=candles labels=labels label="ACME daily prices" format=|v: f64| format!("${v:.2}") />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "candlestick-chart",
    name: "Candlestick chart",
    group: "Charts",
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
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move to the previous or next candle"],
        &["Home / End", "Jump to the first or last candle"],
        &["Escape", "Hide the readout"],
    ],
    props: &[
        &["candles", "Vec<Candle>", "Required. Candle::new(open, high, low, close), optionally .with_volume(v)"],
        &["labels", "Vec<String>", "Required. One label per candle"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Price plot height in px. Default 260"],
        &["volume", "bool", "Show the volume strip when candles have volumes. Default true"],
        &["format", "Fn(f64) -> String", "Formats prices"],
    ],
};
