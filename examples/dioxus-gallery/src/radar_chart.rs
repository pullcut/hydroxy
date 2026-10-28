use dioxus::prelude::*;
use hydroxy_dioxus::radar_chart::{RadarChart, Series};

#[component]
pub fn RadarChartDemo() -> Element {
    let axes: Vec<String> = ["Speed", "Reach", "Reliability", "Support", "Docs", "Value"].map(String::from).to_vec();
    let vendors = vec![
        Series::new("Acme", [82.0, 64.0, 90.0, 55.0, 70.0, 48.0]),
        Series::new("Globex", [58.0, 88.0, 62.0, 80.0, 52.0, 76.0]),
    ];

    rsx! {
        RadarChart { series: vendors, axes, label: "Vendor comparison", max: 100.0 }
    }
}
