use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::waterfall_chart::{Annotation, Step, WaterfallChart};

#[component]
pub fn WaterfallChartDemo() -> Element {
    let bridge = vec![
        Step::start("FY24 revenue", 4_200.0),
        Step::change("New customers", 1_150.0),
        Step::change("Expansion", 640.0),
        Step::change("Price rise", 310.0),
        Step::change("Churn", -820.0),
        Step::change("Discounts", -270.0),
        Step::total("FY25 revenue"),
    ];
    let cash = vec![
        Step::change("Jan", 120.0),
        Step::change("Feb", -45.0),
        Step::change("Mar", 80.0),
        Step::total("Q1"),
        Step::change("Apr", -160.0),
        Step::change("May", 35.0),
        Step::change("Jun", 90.0),
        Step::total("H1"),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Revenue bridge", description: "From last year's revenue to this year's, in thousands.",
                WaterfallChart { steps: bridge, label: "Revenue bridge from FY24 to FY25", format: move |v: f64| format!("£{v:.0}k") }
            }
            ChartFrame { title: "Net cash flow", description: "Monthly changes with quarter and half-year totals.",
                WaterfallChart {
                    steps: cash,
                    label: "Net cash flow by month",
                    height: 200,
                    annotations: vec![Annotation::y_line(100.0, "Reserve floor")],
                }
            }
        }
    }
}
