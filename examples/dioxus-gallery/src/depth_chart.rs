use dioxus::prelude::*;
use hydroxy_dioxus::depth_chart::{DepthChart, Level};

#[component]
pub fn DepthChartDemo() -> Element {
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

    rsx! {
        DepthChart { bids, asks, label: "BTC-USD order book", format: move |v: f64| format!("${v:.1}") }
    }
}
