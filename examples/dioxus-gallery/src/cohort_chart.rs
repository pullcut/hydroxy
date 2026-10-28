use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::cohort_chart::{Cohort, CohortChart};

#[component]
pub fn CohortChartDemo() -> Element {
    let cohorts = vec![
        Cohort::new("Jan 2026", 1_204.0, [1_204.0, 662.0, 518.0, 457.0, 421.0, 398.0, 381.0, 370.0, 362.0]),
        Cohort::new("Feb 2026", 1_118.0, [1_118.0, 637.0, 503.0, 449.0, 413.0, 392.0, 377.0, 366.0]),
        Cohort::new("Mar 2026", 1_356.0, [1_356.0, 801.0, 646.0, 579.0, 538.0, 511.0, 494.0]),
        Cohort::new("Apr 2026", 1_290.0, [1_290.0, 787.0, 640.0, 576.0, 537.0, 514.0]),
        Cohort::new("May 2026", 1_472.0, [1_472.0, 927.0, 762.0, 690.0, 646.0]),
        Cohort::new("Jun 2026", 1_538.0, [1_538.0, 984.0, 818.0, 741.0]),
        Cohort::new("Jul 2026", 1_611.0, [1_611.0, 1_047.0, 880.0]),
        Cohort::new("Aug 2026", 1_704.0, [1_704.0, 1_125.0]),
        Cohort::new("Sep 2026", 1_689.0, [1_689.0]),
    ];
    let periods: Vec<String> = (0..9).map(|m| format!("Month {m}")).collect();

    rsx! {
        ChartFrame {
            title: "Paying customers by signup month",
            description: "Share of each month's signups still paying. Later cohorts hold on better after the March onboarding change.",
            export: true,
            CohortChart { cohorts, periods, label: "Monthly retention by signup month", noun: "customers" }
        }
    }
}
