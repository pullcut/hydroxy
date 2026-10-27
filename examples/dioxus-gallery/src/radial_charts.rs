use dioxus::prelude::*;
use oxide_dioxus::button::{Button, ButtonVariant};
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::radial_chart::{Datum, GaugeChart, RadialBarChart, Threshold};

fn bands() -> Vec<Threshold> {
    vec![
        Threshold::new(0.0, "var(--status-danger)"),
        Threshold::new(70.0, "var(--status-warning)"),
        Threshold::new(90.0, "var(--status-success)"),
    ]
}

#[component]
pub fn RadialChartsDemo() -> Element {
    let goals = vec![
        Datum::new("Interviews", 42.0),
        Datum::new("Surveys", 28.0),
        Datum::new("Tickets", 65.0),
        Datum::new("Reviews", 12.0),
    ];
    // A changed value glides along the arc.
    let mut week = use_signal(|| 0_usize);
    let backlog = [38.0, 64.0, 91.0][week() % 3];

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            ChartFrame { title: "Support SLA met", description: "Share of tickets answered inside the target time.",
                div { style: "display: flex; flex-wrap: wrap; gap: var(--space-8);",
                    GaugeChart {
                        value: 94.0,
                        label: "SLA met this week",
                        caption: "this week",
                        thresholds: bands(),
                        format: move |v: f64| format!("{v:.0}%"),
                    }
                    GaugeChart {
                        value: 78.0,
                        label: "SLA met last week",
                        caption: "last week",
                        thresholds: bands(),
                        format: move |v: f64| format!("{v:.0}%"),
                    }
                    GaugeChart {
                        value: 52.0,
                        label: "SLA met two weeks ago",
                        caption: "two weeks ago",
                        thresholds: bands(),
                        format: move |v: f64| format!("{v:.0}%"),
                    }
                }
            }
            ChartFrame { title: "Backlog cleared", description: "Step through the last three sprints.",
                div { style: "display: flex; align-items: center; flex-wrap: wrap; gap: var(--space-6);",
                    GaugeChart {
                        value: backlog,
                        label: "Backlog cleared this sprint",
                        caption: "of planned work",
                        thresholds: bands(),
                        format: move |v: f64| format!("{v:.0}%"),
                    }
                    Button { variant: ButtonVariant::Secondary, on_click: move |_| week += 1, "Next sprint" }
                }
            }
            ChartFrame { title: "Research goals", description: "Progress against a target of 100 each.",
                RadialBarChart { data: goals, max: 100.0, label: "Progress towards each research goal" }
            }
        }
    }
}
