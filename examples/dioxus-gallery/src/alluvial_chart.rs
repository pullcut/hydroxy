use dioxus::prelude::*;
use hydroxy_dioxus::alluvial_chart::{AlluvialChart, Cohort};
use hydroxy_dioxus::chart_frame::ChartFrame;

#[component]
pub fn AlluvialChartDemo() -> Element {
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].iter().map(|q| format!("{q} 2025")).collect();
    // Accounts that signed up in Q1, by plan at the end of each quarter.
    let accounts = vec![
        Cohort::new(["Free", "Free", "Free", "Free"], 820.0),
        Cohort::new(["Free", "Free", "Starter", "Starter"], 240.0),
        Cohort::new(["Free", "Starter", "Starter", "Pro"], 130.0),
        Cohort::new(["Free", "Starter", "Pro", "Pro"], 90.0),
        Cohort::new(["Free", "Churned", "Churned", "Churned"], 310.0),
        Cohort::new(["Free", "Free", "Churned", "Churned"], 180.0),
        Cohort::new(["Starter", "Starter", "Starter", "Starter"], 260.0),
        Cohort::new(["Starter", "Pro", "Pro", "Pro"], 150.0),
        Cohort::new(["Starter", "Starter", "Churned", "Churned"], 70.0),
        Cohort::new(["Starter", "Free", "Free", "Churned"], 40.0),
        Cohort::new(["Pro", "Pro", "Pro", "Pro"], 210.0),
        Cohort::new(["Pro", "Starter", "Starter", "Starter"], 35.0),
        Cohort::new(["Pro", "Pro", "Churned", "Churned"], 25.0),
    ];

    rsx! {
        ChartFrame {
            title: "Plan moves",
            description: "Accounts that signed up in Q1, by plan at each quarter's end. Most churn leaves from Free in the first quarter.",
            AlluvialChart { data: accounts, steps: quarters, label: "Q1 2025 sign-ups by plan, quarter by quarter" }
        }
    }
}
