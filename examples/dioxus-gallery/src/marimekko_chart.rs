use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::marimekko_chart::{MarimekkoChart, Series};

#[component]
pub fn MarimekkoChartDemo() -> Element {
    let labels = vec!["North America".to_string(), "Europe".into(), "Asia Pacific".into(), "Latin America".into()];
    let series = vec![
        Series::new("Enterprise", [320.0, 180.0, 90.0, 20.0]),
        Series::new("Mid-market", [210.0, 160.0, 120.0, 35.0]),
        Series::new("Small business", [90.0, 110.0, 140.0, 45.0]),
    ];

    rsx! {
        ChartFrame { title: "Market", description: "Column width is each region's share of revenue; height splits it by segment.",
            MarimekkoChart { series, labels, label: "Revenue by region and segment", format: move |v: f64| format!("${v:.0}m") }
        }
    }
}
