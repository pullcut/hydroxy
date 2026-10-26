use dioxus::prelude::*;
use oxide_dioxus::stacked_bar_chart::{Series, StackedBarChart};

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

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            StackedBarChart { series: tickets, labels: weeks, label: "Tickets by channel per week" }
            StackedBarChart { series: headcount, labels: quarters, label: "Headcount change by quarter", height: 180 }
        }
    }
}
