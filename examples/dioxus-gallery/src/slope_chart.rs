use dioxus::prelude::*;
use hydroxy_dioxus::slope_chart::{Series, SlopeChart};

#[component]
pub fn SlopeChartDemo() -> Element {
    let scores = vec![
        Series::new("North", [72.0, 81.0]),
        Series::new("South", [64.0, 61.0]),
        Series::new("East", [58.0, 74.0]),
        Series::new("West", [77.0, 76.0]),
        Series::new("Central", [66.0, 69.0]),
    ];

    rsx! {
        SlopeChart {
            series: scores,
            from_label: "2023",
            to_label: "2024",
            label: "Satisfaction score by region",
            format: move |v: f64| format!("{v:.0}%"),
        }
    }
}
