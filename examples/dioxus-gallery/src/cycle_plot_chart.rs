use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::cycle_plot_chart::{CyclePlotChart, Series};

#[component]
pub fn CyclePlotChartDemo() -> Element {
    let months: Vec<String> =
        ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].map(String::from).to_vec();
    // Monthly rainfall in millimetres, one series per year. Autumn is getting wetter.
    let years = vec![
        Series::new("2021", [88.0, 61.0, 52.0, 41.0, 58.0, 49.0, 47.0, 63.0, 55.0, 79.0, 84.0, 92.0]),
        Series::new("2022", [79.0, 70.0, 49.0, 46.0, 51.0, 54.0, 38.0, 52.0, 64.0, 86.0, 91.0, 88.0]),
        Series::new("2023", [92.0, 58.0, 61.0, 49.0, 47.0, 44.0, 52.0, 58.0, 69.0, 95.0, 97.0, 85.0]),
        Series::new("2024", [84.0, 66.0, 55.0, 52.0, 55.0, 47.0, 41.0, 49.0, 74.0, 101.0, 104.0, 94.0]),
        Series::new("2025", [90.0, 63.0, 50.0, 44.0, 49.0, 39.0, 36.0, 47.0, 78.0, 108.0, 110.0, 97.0]),
    ];
    let days: Vec<String> = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(String::from).to_vec();
    // Average covers per day at a café, one series per week over the last six weeks.
    let weeks = vec![
        Series::new("Week 38", [64.0, 71.0, 69.0, 78.0, 96.0, 142.0, 128.0]),
        Series::new("Week 39", [61.0, 74.0, 72.0, 80.0, 101.0, 150.0, 131.0]),
        Series::new("Week 40", [66.0, 70.0, 75.0, 84.0, 99.0, 139.0, 120.0]),
        Series::new("Week 41", [58.0, 68.0, 77.0, 88.0, 108.0, 155.0, 126.0]),
        Series::new("Week 42", [63.0, 72.0, 80.0, 91.0, 112.0, 161.0, 133.0]),
        Series::new("Week 43", [60.0, 75.0, 83.0, 95.0, 118.0, 166.0, 129.0]),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame {
                title: "Monthly rainfall, 2021 to 2025",
                description: "Each month's line runs through its five years, oldest first; the flat rule is the five-year mean.",
                CyclePlotChart {
                    periods: months,
                    cycles: years,
                    label: "Monthly rainfall in millimetres, 2021 to 2025",
                    format: move |v: f64| format!("{v:.0} mm"),
                }
            }
            ChartFrame { title: "Covers by weekday", description: "Average covers per day over the last six weeks. Weekends are growing fastest.",
                CyclePlotChart { periods: days, cycles: weeks, label: "Café covers by weekday, weeks 38 to 43", height: 220 }
            }
        }
    }
}
