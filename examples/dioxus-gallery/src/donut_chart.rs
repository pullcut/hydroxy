use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::donut_chart::{Datum, DonutChart};

#[component]
pub fn DonutChartDemo() -> Element {
    let channels = vec![
        Datum::new("Email", 182.0),
        Datum::new("Chat", 124.0),
        Datum::new("Phone", 61.0),
        Datum::new("Forms", 43.0),
    ];
    let sources = vec![
        Datum::new("Interviews", 12.0),
        Datum::new("Support tickets", 9.0),
        Datum::new("Surveys", 7.0),
        Datum::new("Sales notes", 5.0),
        Datum::new("NPS", 4.0),
        Datum::new("App reviews", 2.0),
        Datum::new("Forums", 1.0),
        Datum::new("Social", 1.0),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Tickets", description: "By channel, this quarter.",
                DonutChart { data: channels, label: "Tickets by channel", center_label: "Tickets" }
            }
            ChartFrame { title: "Sources", description: "More than six fold into a grey Other.",
                DonutChart { data: sources, label: "Sources by type", center_label: "Sources", size: 150, thickness: 18 }
            }
        }
    }
}
