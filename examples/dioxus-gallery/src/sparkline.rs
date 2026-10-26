use dioxus::prelude::*;
use oxide_dioxus::sparkline::{Curve, Sparkline};

#[component]
pub fn SparklineDemo() -> Element {
    let signups = vec![12.0, 15.0, 11.0, 18.0, 22.0, 19.0, 25.0, 24.0, 29.0, 31.0, 28.0, 34.0];
    let churn = vec![4.1, 3.8, 4.4, 4.0, 3.6, 3.9, 3.2, 3.4, 3.0, 2.8];

    rsx! {
        div { style: "display: grid; gap: var(--space-6);",
            p { style: "margin: 0; font: var(--text-md); color: var(--text-2);",
                "Signups this quarter "
                Sparkline { values: signups.clone(), label: "Signups this quarter" }
                " are up 183%."
            }
            span { style: "--sparkline-color: var(--chart-4);",
                Sparkline { values: signups, label: "Signups, filled", width: 160, height: 40, area: true, curve: Curve::Monotone }
            }
            span { style: "--sparkline-color: var(--chart-2);",
                Sparkline {
                    values: churn,
                    label: "Monthly churn",
                    width: 160,
                    height: 40,
                    format: move |v: f64| format!("{v:.1}%"),
                }
            }
        }
    }
}
