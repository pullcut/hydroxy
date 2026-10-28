use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::funnel_chart::{Datum, FunnelChart, Orientation};

#[component]
pub fn FunnelChartDemo() -> Element {
    let signup = vec![
        Datum::new("Visited pricing", 18_400.0),
        Datum::new("Started trial", 4_120.0),
        Datum::new("Invited a teammate", 1_870.0),
        Datum::new("Connected data", 1_240.0),
        Datum::new("Paid", 610.0),
    ];
    let hiring = vec![
        Datum::new("Applied", 640.0),
        Datum::new("Screened", 210.0),
        Datum::new("Interviewed", 72.0),
        Datum::new("Offered", 14.0),
        Datum::new("Hired", 11.0),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Signup funnel", description: "Last 30 days, with conversion between stages.",
                FunnelChart { data: signup, label: "Signup funnel" }
            }
            ChartFrame { title: "Hiring", description: "Engineering roles this year, left to right.",
                FunnelChart { data: hiring, label: "Hiring funnel", orientation: Orientation::Horizontal, height: 200 }
            }
        }
    }
}
