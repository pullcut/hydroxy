use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::dot_plot_chart::{Annotation, DotPlotChart, Series};

#[component]
pub fn DotPlotChartDemo() -> Element {
    let roles: Vec<String> =
        ["Product designer", "Frontend engineer", "Data analyst", "Support lead", "Engineering manager"].map(String::from).to_vec();
    let salaries = vec![
        Series::new("2024", [58.0, 64.0, 49.0, 41.0, 88.0]),
        Series::new("2025", [61.0, 70.0, 53.0, 42.5, 91.0]),
    ];
    let cities: Vec<String> = ["Lisbon", "Berlin", "Toronto", "Melbourne"].map(String::from).to_vec();
    let rent = vec![Series::new("Median rent", [1_350.0, 1_480.0, 2_150.0, 1_920.0])];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Median salary by role", description: "Thousands per year, last year against this year.",
                DotPlotChart { rows: roles, series: salaries, label: "Median salary by role, 2024 and 2025", format: move |v: f64| format!("£{v:.0}k") }
            }
            ChartFrame { title: "One-bedroom rent", description: "A single series is a plain dot plot.",
                DotPlotChart {
                    rows: cities,
                    series: rent,
                    label: "Median one-bedroom rent by city",
                    annotations: vec![Annotation::y_line(1_800.0, "Budget")],
                }
            }
        }
    }
}
