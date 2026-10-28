use dioxus::prelude::*;
use hydroxy_dioxus::candlestick_chart::{Annotation, Candle, CandlestickChart};

#[component]
pub fn CandlestickChartDemo() -> Element {
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

    rsx! {
        // Scroll or pinch to zoom, drag to pan, or press + and − on the focused plot.
        CandlestickChart {
            candles,
            labels,
            label: "ACME daily prices",
            format: move |v: f64| format!("${v:.2}"),
            annotations: levels,
            zoom: true,
        }
    }
}
