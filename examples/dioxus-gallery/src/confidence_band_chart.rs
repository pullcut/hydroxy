use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::confidence_band_chart::{Annotation, ConfidenceBandChart, Interval};

#[component]
pub fn ConfidenceBandChartDemo() -> Element {
    // Conversion rate (%) for a new checkout, with a 95% interval that narrows as traffic builds.
    let rate = [2.4, 2.9, 3.1, 3.0, 3.3, 3.2, 3.4, 3.5, 3.4, 3.6];
    let spread = [0.9, 0.7, 0.6, 0.5, 0.45, 0.4, 0.35, 0.32, 0.3, 0.28];
    let weeks: Vec<Interval> = rate
        .iter()
        .zip(spread)
        .enumerate()
        .map(|(i, (&v, s))| Interval::new(format!("Week {}", i + 1), v, v - s, v + s))
        .collect();

    rsx! {
        ChartFrame { title: "New checkout conversion", description: "Weekly estimate with its 95% interval. The dashed line is the old checkout's rate.",
            ConfidenceBandChart {
                data: weeks,
                label: "Weekly conversion rate of the new checkout, with 95% interval",
                value_label: "Conversion",
                format: move |v: f64| format!("{v:.1}%"),
                annotations: vec![Annotation::y_line(2.8, "Old checkout")],
            }
        }
    }
}
