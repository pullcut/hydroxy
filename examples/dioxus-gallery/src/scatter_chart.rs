use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::scatter_chart::{Group, Point, ScatterChart};

#[component]
pub fn ScatterChartDemo() -> Element {
    let teams = vec![
        Group::new(
            "Research",
            vec![
                Point::new(4.0, 62.0),
                Point::new(6.0, 68.0),
                Point::new(7.0, 71.0),
                Point::new(9.0, 77.0),
                Point::new(12.0, 80.0),
                Point::new(14.0, 86.0),
            ],
        ),
        Group::new(
            "Support",
            vec![
                Point::new(3.0, 48.0),
                Point::new(5.0, 55.0),
                Point::new(8.0, 52.0),
                Point::new(10.0, 63.0),
                Point::new(13.0, 66.0),
                Point::new(15.0, 70.0),
            ],
        ),
    ];
    let accounts = vec![Group::new(
        "Accounts",
        vec![
            Point::sized(12.0, 3_400.0, 8.0),
            Point::sized(28.0, 5_900.0, 34.0),
            Point::sized(41.0, 4_100.0, 15.0),
            Point::sized(55.0, 9_800.0, 62.0),
            Point::sized(63.0, 7_200.0, 24.0),
            Point::sized(80.0, 12_600.0, 90.0),
        ],
    )];

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            ChartFrame {
                title: "Interviews against satisfaction",
                description: "One dot per person, by how many interviews they read.",
                ScatterChart {
                    groups: teams,
                    label: "Interviews read against satisfaction score",
                    x_label: "Interviews",
                    y_label: "Satisfaction",
                }
            }
            ChartFrame { title: "Revenue by account", description: "Bubble area is seats.",
                ScatterChart {
                    groups: accounts,
                    label: "Revenue against tenure, sized by seats",
                    x_label: "Months",
                    y_label: "Revenue",
                    size_label: "Seats",
                    height: 240,
                    format_y: move |v: f64| format!("£{:.0}k", v / 1000.0),
                }
            }
        }
    }
}
