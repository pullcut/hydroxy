use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::nested_donut_chart::{Datum, NestedDonutChart, series};

#[component]
pub fn NestedDonutChartDemo() -> Element {
    // Website sessions by source, in thousands, one ring per year.
    let year = |name: &str, values: [f64; 5]| {
        let sources = ["Search", "Direct", "Social", "Referral", "Email"];
        (name.to_string(), sources.iter().zip(values).map(|(s, v)| Datum::new(*s, v)).collect::<Vec<_>>())
    };
    let rings = series(&[year("2023", [412.0, 186.0, 94.0, 61.0, 38.0]), year("2024", [455.0, 204.0, 141.0, 58.0, 47.0]), year("2025", [468.0, 231.0, 212.0, 52.0, 66.0])]);

    rsx! {
        ChartFrame {
            title: "Sessions by source",
            description: "Thousands of sessions a year, 2023 in the middle and 2025 outside. Social has more than doubled its share.",
            NestedDonutChart { rings, label: "Website sessions by source, 2023 to 2025", format: |v: f64| format!("{v:.0}k") }
        }
    }
}
