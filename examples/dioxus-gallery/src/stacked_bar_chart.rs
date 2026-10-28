use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::stacked_bar_chart::{Annotation, Orientation, Series, StackedBarChart};

#[component]
pub fn StackedBarChartDemo() -> Element {
    let weeks: Vec<String> = ["30 Sep", "7 Oct", "14 Oct", "21 Oct"].map(String::from).to_vec();
    let tickets = vec![
        Series::new("Email", [42.0, 38.0, 51.0, 47.0]),
        Series::new("Chat", [28.0, 35.0, 31.0, 40.0]),
        Series::new("Phone", [12.0, 9.0, 14.0, 11.0]),
    ];
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].map(String::from).to_vec();
    let headcount = vec![
        Series::new("Hires", [8.0, 5.0, 11.0, 6.0]),
        Series::new("Transfers in", [2.0, 3.0, 1.0, 2.0]),
        Series::new("Leavers", [-3.0, -6.0, -2.0, -4.0]),
    ];

    let teams: Vec<String> =
        ["Customer research", "Support operations", "Sales enablement", "Product design"].map(String::from).to_vec();
    let spend = vec![
        Series::new("People", [42.0, 31.0, 24.0, 38.0]),
        Series::new("Tools", [9.0, 14.0, 11.0, 7.0]),
        Series::new("Travel", [5.0, 2.0, 12.0, 3.0]),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Tickets", description: "By channel, per week.",
                StackedBarChart { series: tickets.clone(), labels: weeks.clone(), label: "Tickets by channel per week" }
            }
            // The same data as shares: every column reaches 100%.
            ChartFrame { title: "Channel mix", description: "Each channel's share of the week's tickets.",
                StackedBarChart {
                    series: tickets,
                    labels: weeks,
                    label: "Share of tickets by channel per week",
                    normalize: true,
                    show_totals: false,
                    annotations: vec![Annotation::y_line(50.0, "Half")],
                }
            }
            ChartFrame { title: "Headcount", description: "Hires and leavers; the total is the net change.",
                StackedBarChart { series: headcount, labels: quarters, label: "Headcount change by quarter", height: 180 }
            }
            ChartFrame { title: "Spend by team", description: "Horizontal stacks keep long names readable.",
                StackedBarChart {
                    series: spend,
                    labels: teams,
                    label: "Spend by team and kind",
                    orientation: Orientation::Horizontal,
                    height: 180,
                }
            }
        }
    }
}
