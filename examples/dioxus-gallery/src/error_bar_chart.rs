use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::error_bar_chart::{Annotation, ErrorBarChart, Interval};

#[component]
pub fn ErrorBarChartDemo() -> Element {
    let variants = vec![
        Interval::new("Control", 4.1, 3.6, 4.6),
        Interval::new("Shorter form", 4.9, 4.3, 5.5),
        Interval::new("Social proof", 4.4, 3.8, 5.0),
        Interval::new("Free trial", 5.6, 4.8, 6.4),
    ];
    let teams = vec![
        Interval::new("Platform", 18.0, 14.5, 22.0),
        Interval::new("Payments", 26.0, 21.0, 31.5),
        Interval::new("Mobile", 21.5, 16.0, 28.0),
        Interval::new("Growth", 12.0, 9.5, 15.0),
        Interval::new("Data", 15.5, 12.0, 19.5),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Signup conversion", description: "Each variant with its 95% confidence interval.",
                ErrorBarChart {
                    data: variants,
                    label: "Signup conversion by variant",
                    format: move |v: f64| format!("{v:.1}%"),
                    annotations: vec![Annotation::y_line(4.1, "Control")],
                }
            }
            ChartFrame { title: "Cycle time", description: "Median days from start to merge, with the interquartile range.",
                ErrorBarChart { data: teams, label: "Cycle time by team in days", bars: true, height: 220 }
            }
        }
    }
}
