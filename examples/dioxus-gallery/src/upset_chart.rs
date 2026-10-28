use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::upset_chart::{Intersection, UpsetChart};

#[component]
pub fn UpsetChartDemo() -> Element {
    // Customers by the exact set of notification channels they keep switched on.
    let channels = vec![
        Intersection::new(["Email"], 4120.0),
        Intersection::new(["Email", "Push"], 2860.0),
        Intersection::new(["Push"], 1940.0),
        Intersection::new(["Email", "In-app"], 1310.0),
        Intersection::new(["Email", "Push", "In-app"], 1180.0),
        Intersection::new(["In-app"], 760.0),
        Intersection::new(["Email", "SMS"], 640.0),
        Intersection::new(["Push", "In-app"], 590.0),
        Intersection::new(["Email", "Slack"], 470.0),
        Intersection::new(["SMS"], 380.0),
        Intersection::new(["Email", "Push", "SMS"], 310.0),
        Intersection::new(["Slack"], 260.0),
        Intersection::new(["Email", "Slack", "In-app"], 220.0),
        Intersection::new(["Push", "SMS"], 140.0),
        Intersection::new(["Email", "Push", "In-app", "Slack"], 90.0),
    ];

    rsx! {
        ChartFrame {
            title: "Notification channels",
            description: "Customers by the exact combination of channels they keep switched on. Email alone is still the largest group.",
            UpsetChart { data: channels, label: "Customers by notification channels switched on" }
        }
    }
}
