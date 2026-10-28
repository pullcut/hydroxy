use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::clock_chart::{ClockChart, Series};

#[component]
pub fn ClockChartDemo() -> Element {
    // Support chats started in each hour, averaged over October.
    let weekdays = Series::new(
        "Weekdays",
        [3.0, 2.0, 1.0, 1.0, 2.0, 4.0, 9.0, 21.0, 38.0, 46.0, 44.0, 40.0, 35.0, 39.0, 42.0, 37.0, 31.0, 24.0, 17.0, 13.0, 11.0, 9.0, 6.0, 4.0],
    );
    let weekends = Series::new(
        "Weekends",
        [6.0, 4.0, 2.0, 1.0, 1.0, 1.0, 2.0, 5.0, 9.0, 15.0, 20.0, 24.0, 26.0, 25.0, 23.0, 22.0, 21.0, 20.0, 19.0, 18.0, 17.0, 14.0, 11.0, 8.0],
    );
    // Steps by hour on one Saturday.
    let steps = Series::new(
        "Steps",
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 120.0, 840.0, 2_310.0, 1_460.0, 980.0, 1_720.0, 640.0, 3_850.0, 4_120.0, 1_210.0, 690.0, 1_030.0, 410.0, 260.0, 180.0, 40.0, 0.0],
    );

    rsx! {
        div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: var(--space-10);",
            ChartFrame {
                title: "Support chats by hour",
                description: "Chats started per hour, averaged over October. Weekdays peak mid-morning; weekends spread out.",
                ClockChart { series: vec![weekdays, weekends], label: "Support chats started by hour, weekdays and weekends" }
            }
            ChartFrame { title: "Steps on Saturday", description: "One series, hour by hour. Hover an hour or focus the clock and use the arrow keys.",
                ClockChart { series: vec![steps], label: "Steps by hour on Saturday", size: 260, format: move |v: f64| format!("{v:.0}") }
            }
        }
    }
}
