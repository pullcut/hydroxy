use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::survival_chart::{SurvivalChart, SurvivalGroup};

#[component]
pub fn SurvivalChartDemo() -> Element {
    // Weeks in remission from the 6-MP leukaemia trial (Freireich et al., 1963).
    // `false` marks a patient still in remission when last seen (censored).
    let drug = SurvivalGroup::new(
        "6-MP",
        [
            (6.0, true), (6.0, true), (6.0, true), (6.0, false), (7.0, true), (9.0, false), (10.0, true), (10.0, false), (11.0, false),
            (13.0, true), (16.0, true), (17.0, false), (19.0, false), (20.0, false), (22.0, true), (23.0, true), (25.0, false),
            (32.0, false), (32.0, false), (34.0, false), (35.0, false),
        ],
    );
    let placebo = SurvivalGroup::new(
        "Placebo",
        [1.0, 1.0, 2.0, 2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 8.0, 8.0, 8.0, 8.0, 11.0, 11.0, 12.0, 12.0, 15.0, 17.0, 22.0, 23.0].map(|w| (w, true)),
    );

    rsx! {
        ChartFrame { title: "Remission by treatment", description: "Share of patients still in remission, with 95% intervals. Ticks mark patients last seen in remission.",
            SurvivalChart {
                data: vec![drug, placebo],
                label: "Kaplan-Meier curves of weeks in remission",
                time_label: "Week",
                show_ci: true,
                risk_table: true,
            }
        }
    }
}
