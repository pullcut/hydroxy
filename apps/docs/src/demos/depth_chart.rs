use hydroxy_leptos::depth_chart::{DepthChart, Level};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let bids = vec![
        Level::new(64243.1, 1.32),
        Level::new(64237.6, 1.68),
        Level::new(64232.8, 1.73),
        Level::new(64229.9, 1.17),
        Level::new(64222.2, 1.51),
        Level::new(64217.3, 0.84),
        Level::new(64213.6, 1.11),
        Level::new(64208.4, 1.64),
        Level::new(64205.0, 1.22),
        Level::new(64199.2, 2.28),
        Level::new(64192.7, 1.30),
        Level::new(64187.6, 1.35),
    ];
    let asks = vec![
        Level::new(64256.9, 0.38),
        Level::new(64260.0, 1.50),
        Level::new(64265.6, 0.66),
        Level::new(64272.9, 1.66),
        Level::new(64275.9, 1.87),
        Level::new(64281.6, 1.55),
        Level::new(64285.6, 2.00),
        Level::new(64292.1, 2.11),
        Level::new(64297.7, 1.26),
        Level::new(64301.1, 1.15),
        Level::new(64305.4, 1.09),
        Level::new(64310.9, 1.92),
    ];

    view! { <DepthChart bids=bids asks=asks label="BTC-USD order book" format=|v: f64| format!("${v:.1}") /> }
        .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "depth-chart",
    name: "Depth chart",
    group: "Finance",
    summary: "An order book as two step areas: bids stepping down to the left of the spread and asks stepping up to the right, each the total size available up to that price. A dashed line marks the mid price.",
    demo,
    leptos: include_str!("depth_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/depth_chart.rs"),
    parts: &[
        &["DepthChart", "root", "figure with a screen-reader table of every level"],
        &["", "overlay", "Focusable plot; arrow keys move between price levels"],
        &["", "area", "Cumulative bid or ask steps, data-side bid or ask"],
        &["", "mid", "Dashed line at the mid price, with its label"],
        &["", "tooltip", "Price, cumulative depth and size at the level"],
        &["", "annotation", "Reference line, band or point"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous price level"],
        &["Home / End", "Lowest or highest price"],
        &["Escape", "Clear"],
    ],
    props: &[
        &["bids", "Vec<Level>", "Required. Level::new(price, size), any order"],
        &["asks", "Vec<Level>", "Required. Level::new(price, size), any order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["annotations", "Vec<Annotation>", "X in prices, y in cumulative size"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats prices; sizes are shortened, such as 12.5k"],
    ],
};
