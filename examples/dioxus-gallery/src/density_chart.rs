use dioxus::prelude::*;
use hydroxy_dioxus::density_chart::{DensityChart, Group};

#[component]
pub fn DensityChartDemo() -> Element {
    let groups = vec![
        Group::new(
            "Web",
            [
                42.0, 55.0, 38.0, 61.0, 47.0, 72.0, 50.0, 44.0, 66.0, 58.0, 39.0, 81.0, 53.0, 49.0, 95.0, 57.0, 46.0,
                63.0, 52.0, 70.0, 41.0, 59.0, 48.0, 120.0, 54.0,
            ],
        ),
        Group::new(
            "Mobile app",
            [
                28.0, 35.0, 31.0, 26.0, 40.0, 33.0, 29.0, 37.0, 24.0, 45.0, 32.0, 30.0, 27.0, 38.0, 34.0, 22.0, 41.0,
                36.0, 29.0, 31.0, 55.0, 33.0, 27.0, 30.0, 35.0,
            ],
        ),
        Group::new(
            "Retail",
            [
                85.0, 110.0, 95.0, 130.0, 78.0, 102.0, 145.0, 90.0, 118.0, 99.0, 87.0, 125.0, 108.0, 160.0, 93.0,
                115.0, 82.0, 105.0, 138.0, 97.0, 112.0, 88.0, 175.0, 101.0, 94.0,
            ],
        ),
    ];

    rsx! {
        DensityChart { groups, label: "Order value by channel", format: move |v: f64| format!("${v:.0}") }
    }
}
