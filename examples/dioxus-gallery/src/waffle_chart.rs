use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::waffle_chart::{Datum, WaffleChart};

#[component]
pub fn WaffleChartDemo() -> Element {
    let spend = vec![
        Datum::new("Salaries", 5_400.0),
        Datum::new("Cloud", 1_900.0),
        Datum::new("Marketing", 1_350.0),
        Datum::new("Office", 750.0),
        Datum::new("Other", 600.0),
    ];
    let seats = vec![Datum::new("Active", 312.0), Datum::new("Invited", 41.0)];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Spend", description: "Each square is 1% of this year's budget.",
                WaffleChart { data: spend, label: "Spend by category", format: move |v: f64| format!("${:.1}m", v / 1000.0) }
            }
            ChartFrame { title: "Seats", description: "Of 500 licensed; empty squares are unused.",
                WaffleChart { data: seats, label: "Seats in use", total: 500.0 }
            }
        }
    }
}
