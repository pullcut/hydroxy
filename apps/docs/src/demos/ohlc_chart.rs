use leptos::prelude::*;
use oxide_leptos::ohlc_chart::{Annotation, Candle, OhlcChart};

pub fn demo() -> AnyView {
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

    view! {
        <OhlcChart
            candles=candles
            labels=labels
            label="ACME daily, October"
            annotations=vec![Annotation::y_line(155.0, "Resistance")]
            format=|v: f64| format!("${v:.2}")
        />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "ohlc-chart",
    name: "OHLC chart",
    group: "Finance",
    summary: "Open, high, low and close per period as a thin bar with the open ticked left and the close ticked right. Lighter than candlesticks when there are many periods.",
    demo,
    leptos: include_str!("ohlc_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/ohlc_chart.rs"),
    parts: &[
        &["OhlcChart", "root", "figure with a screen-reader table of every period"],
        &["", "overlay", "Focusable plot; arrow keys move between periods"],
        &["", "bar", "One per period, data-dir up or down"],
        &["", "range", "Low to high"],
        &["", "open", "Tick to the left at the open"],
        &["", "close", "Tick to the right at the close"],
        &["", "annotation", "Reference line, band or point"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous period"],
        &["Home / End", "First or last period"],
        &["Escape", "Clear"],
    ],
    props: &[
        &["candles", "Vec<Candle>", "Required. Candle::new(open, high, low, close), oldest first"],
        &["labels", "Vec<String>", "Required. One per period, such as a date"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["scale", "ScaleKind", "Linear, Log or Symlog price axis. Default Linear"],
        &["annotations", "Vec<Annotation>", "Y in prices, x in period indices"],
        &["height", "u16", "Plot height in px. Default 260"],
        &["format", "Fn(f64) -> String", "Formats prices on the axis, tooltip and table"],
    ],
};
