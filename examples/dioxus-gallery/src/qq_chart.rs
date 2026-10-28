use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::qq_chart::QQChart;

#[component]
pub fn QQChartDemo() -> Element {
    // Checkout latency in ms: roughly normal, so the points follow the line.
    let latency = vec![
        181.0, 236.0, 214.0, 198.0, 229.0, 251.0, 205.0, 222.0, 190.0, 244.0, 217.0, 208.0, 263.0, 226.0, 199.0, 232.0,
        211.0, 186.0, 248.0, 220.0, 203.0, 238.0, 215.0, 227.0, 194.0, 257.0, 209.0, 231.0, 223.0, 212.0,
    ];
    // Session length in minutes: skewed right, so the top end curves away above the line.
    let sessions = vec![
        1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0, 6.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 13.0, 15.0,
        17.0, 20.0, 24.0, 29.0, 35.0, 44.0, 58.0,
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10); grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));",
            ChartFrame { title: "Checkout latency", description: "Close to normal: the points hug the line.",
                QQChart { sample: latency, label: "Normal Q-Q plot of checkout latency", value_label: "Latency", format: move |v: f64| format!("{v:.0} ms") }
            }
            ChartFrame { title: "Session length", description: "Skewed right: long sessions bend up and away from the line.",
                QQChart { sample: sessions, label: "Normal Q-Q plot of session length", value_label: "Session", format: move |v: f64| format!("{v:.0} min") }
            }
        }
    }
}
