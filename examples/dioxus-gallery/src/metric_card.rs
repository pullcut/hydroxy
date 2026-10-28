use dioxus::prelude::*;
use hydroxy_dioxus::metric_card::MetricCard;

#[component]
pub fn MetricCardDemo() -> Element {
    let weeks: Vec<String> = (1..=12).map(|w| format!("Week {w}")).collect();
    let revenue = vec![38.2, 39.0, 40.1, 39.4, 41.8, 42.5, 43.0, 42.2, 44.9, 46.1, 45.3, 48.4];
    let latency = vec![212.0, 205.0, 220.0, 198.0, 190.0, 186.0, 192.0, 181.0, 176.0, 179.0, 171.0, 168.0];
    let errors = vec![0.42, 0.38, 0.40, 0.44, 0.51, 0.48, 0.55, 0.61, 0.58, 0.66, 0.63, 0.71];

    rsx! {
        div { style: "display: grid; gap: var(--space-4); grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));",
            MetricCard {
                label: "Weekly revenue",
                value: 48.4,
                previous: Some(45.3),
                comparison: "vs last week",
                trend: revenue,
                trend_labels: weeks.clone(),
                format: move |v: f64| format!("${v:.1}k"),
            }
            MetricCard {
                label: "p95 latency",
                value: 168.0,
                previous: Some(171.0),
                unit: "ms",
                comparison: "vs last week",
                higher_is_better: false,
                trend: latency,
                trend_labels: weeks.clone(),
            }
            MetricCard {
                label: "Error rate",
                value: 0.71,
                previous: Some(0.63),
                unit: "%",
                comparison: "vs last week",
                higher_is_better: false,
                trend: errors,
                trend_labels: weeks,
                format: move |v: f64| format!("{v:.2}"),
            }
        }
    }
}
