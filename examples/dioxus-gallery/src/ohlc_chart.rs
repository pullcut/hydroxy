use dioxus::prelude::*;
use oxide_dioxus::ohlc_chart::{Annotation, Candle, OhlcChart};

#[component]
pub fn OhlcChartDemo() -> Element {
    let candles = vec![
        Candle::new(148.00, 148.56, 147.39, 147.64),
        Candle::new(147.64, 148.66, 146.91, 147.14),
        Candle::new(147.14, 150.26, 146.00, 149.79),
        Candle::new(149.79, 150.97, 149.59, 150.54),
        Candle::new(150.54, 151.48, 146.52, 147.07),
        Candle::new(147.07, 150.23, 145.15, 148.37),
        Candle::new(148.37, 148.88, 146.28, 146.61),
        Candle::new(146.61, 147.28, 145.91, 146.71),
        Candle::new(146.71, 148.02, 145.98, 147.59),
        Candle::new(147.59, 152.18, 146.27, 151.57),
        Candle::new(151.57, 152.38, 150.03, 150.40),
        Candle::new(150.40, 151.10, 150.10, 150.37),
        Candle::new(150.37, 151.42, 149.01, 149.59),
        Candle::new(149.59, 153.36, 149.32, 152.47),
        Candle::new(152.47, 155.25, 152.42, 153.61),
        Candle::new(153.61, 158.90, 153.26, 156.68),
        Candle::new(156.68, 157.58, 156.10, 156.65),
        Candle::new(156.65, 158.32, 155.74, 156.71),
        Candle::new(156.71, 159.43, 155.13, 158.39),
        Candle::new(158.39, 159.51, 156.96, 159.38),
    ];
    let labels: Vec<String> = [1, 2, 5, 6, 7, 8, 9, 12, 13, 14, 15, 16, 19, 20, 21, 22, 23, 26, 27, 28].map(|d| format!("{d} Oct")).to_vec();

    rsx! {
        OhlcChart {
            candles,
            labels,
            label: "ACME daily, October",
            annotations: vec![Annotation::y_line(155.0, "Resistance")],
            format: move |v: f64| format!("${v:.2}"),
        }
    }
}
