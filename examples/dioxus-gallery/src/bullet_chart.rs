use dioxus::prelude::*;
use oxide_dioxus::bullet_chart::{BulletChart, BulletRow};
use oxide_dioxus::chart_frame::ChartFrame;

#[component]
pub fn BulletChartDemo() -> Element {
    let rows = vec![
        BulletRow::new("Revenue ($k)", 271.0).target(250.0).ranges([150.0, 225.0, 300.0]),
        BulletRow::new("Profit (%)", 22.5).target(26.0).ranges([20.0, 25.0, 30.0]),
        BulletRow::new("New customers", 1_640.0).target(2_000.0).ranges([1_400.0, 2_000.0, 2_500.0]),
        BulletRow::new("Satisfaction", 4.4).target(4.5).ranges([3.5, 4.25, 5.0]),
    ];

    rsx! {
        ChartFrame { title: "Q3 scorecard", description: "Each measure on its own scale, against target.",
            BulletChart { rows, label: "Q3 scorecard", range_labels: vec!["Poor".to_string(), "Fair".into(), "Good".into()] }
        }
    }
}
