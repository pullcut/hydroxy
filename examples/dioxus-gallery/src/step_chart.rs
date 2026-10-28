use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::step_chart::{Series, StepChart, StepEvent, StepMode};

#[component]
pub fn StepChartDemo() -> Element {
    let months: Vec<String> = (2022..=2024).flat_map(|y| ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].map(|m| format!("{m} {}", y % 100))).collect();
    // The rate set at each month's meeting; it holds until the next change.
    let rate = vec![
        0.25, 0.5, 0.75, 0.75, 1.0, 1.25, 1.25, 1.75, 2.25, 2.25, 3.0, 3.5,
        3.5, 4.0, 4.25, 4.25, 4.5, 5.0, 5.0, 5.25, 5.25, 5.25, 5.25, 5.25,
        5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.25, 5.0, 5.0, 5.0, 4.75, 4.75,
    ];
    let events = vec![StepEvent::new(1, "First rise"), StepEvent::new(19, "Peak at 5.25%"), StepEvent::new(31, "First cut")];
    let quarters: Vec<String> = ["Q1 25", "Q2 25", "Q3 25", "Q4 25", "Q1 26", "Q2 26", "Q3 26", "Q4 26"].map(String::from).to_vec();
    let seats = vec![
        Series::new("Engineering", [42.0, 42.0, 48.0, 55.0, 55.0, 61.0, 61.0, 64.0]),
        Series::new("Support", [18.0, 22.0, 22.0, 22.0, 26.0, 26.0, 30.0, 30.0]),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Base rate", description: "The policy rate after each monthly decision, January 2022 to December 2024.",
                StepChart {
                    labels: months,
                    series: vec![Series::new("Base rate", rate)],
                    events,
                    label: "Base rate by month",
                    format: move |v: f64| format!("{v:.2}%"),
                }
            }
            // Headcount changes when a quarter closes, so the step lands before each point.
            ChartFrame { title: "Headcount by team", description: "People employed at the end of each quarter.",
                StepChart { labels: quarters, series: seats, mode: StepMode::Before, label: "Headcount by team and quarter", height: 200 }
            }
        }
    }
}
