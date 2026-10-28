use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::grouped_bar_chart::{Annotation, GroupedBarChart, Orientation, Series};

#[component]
pub fn GroupedBarChartDemo() -> Element {
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].map(String::from).to_vec();
    let signups = vec![
        Series::new("Organic", [1_240.0, 1_410.0, 1_380.0, 1_620.0]),
        Series::new("Paid", [860.0, 1_020.0, 1_190.0, 1_080.0]),
        Series::new("Referral", [310.0, 360.0, 455.0, 520.0]),
    ];
    let regions: Vec<String> =
        ["North America", "Western Europe", "Asia Pacific", "Latin America"].map(String::from).to_vec();
    let margin = vec![Series::new("2024", [18.0, 12.5, 6.0, -4.5]), Series::new("2025", [21.0, 14.0, 9.5, 2.0])];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Signups by channel", description: "Each quarter, side by side.",
                GroupedBarChart {
                    categories: quarters,
                    series: signups,
                    label: "Signups by channel per quarter",
                    annotations: vec![Annotation::y_line(1_500.0, "Organic target")],
                }
            }
            ChartFrame { title: "Operating margin", description: "By region, this year against last.",
                GroupedBarChart {
                    categories: regions,
                    series: margin,
                    label: "Operating margin by region",
                    orientation: Orientation::Horizontal,
                    height: 260,
                    format: move |v: f64| format!("{v:.0}%"),
                }
            }
        }
    }
}
