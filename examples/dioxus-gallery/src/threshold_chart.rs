use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::threshold_chart::{Annotation, Curve, Datum, ThresholdChart};

#[component]
pub fn ThresholdChartDemo() -> Element {
    // p95 API latency over two weeks, in milliseconds.
    let latency = vec![
        Datum::new("Oct 1", 240.0),
        Datum::new("Oct 2", 262.0),
        Datum::new("Oct 3", 251.0),
        Datum::new("Oct 4", 298.0),
        Datum::new("Oct 5", 334.0),
        Datum::new("Oct 6", 356.0),
        Datum::new("Oct 7", 312.0),
        Datum::new("Oct 8", 276.0),
        Datum::new("Oct 9", 259.0),
        Datum::new("Oct 10", 288.0),
        Datum::new("Oct 11", 321.0),
        Datum::new("Oct 12", 305.0),
        Datum::new("Oct 13", 270.0),
        Datum::new("Oct 14", 248.0),
    ];

    rsx! {
        ChartFrame { title: "API latency", description: "Daily p95 against the 300 ms objective. Days over it show in the first colour.",
            ThresholdChart {
                data: latency,
                threshold: 300.0,
                label: "Daily p95 API latency against the 300 ms objective",
                threshold_label: "Objective",
                above_label: "Over objective",
                below_label: "Within objective",
                curve: Curve::Monotone,
                format: move |v: f64| format!("{v:.0} ms"),
                annotations: vec![Annotation::x_band(4.0, 6.0, "Incident")],
            }
        }
    }
}
