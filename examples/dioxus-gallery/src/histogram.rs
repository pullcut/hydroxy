use dioxus::prelude::*;
use hydroxy_dioxus::histogram::{Annotation, Histogram};

#[component]
pub fn HistogramDemo() -> Element {
    // A right-skewed spread of response times: most are quick, a few are slow.
    let times: Vec<f64> = (0..160)
        .map(|i| {
            let t = (f64::from(i) * 0.618_034).fract();
            60.0 + 340.0 * t * t * t + f64::from(i % 7) * 4.0
        })
        .collect();

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            Histogram {
                values: times.clone(),
                label: "Response time",
                format: move |v: f64| format!("{v:.0} ms"),
                annotations: vec![Annotation::x_line(250.0, "Target")],
            }
            Histogram { values: times, label: "Response time, 10 bins", bins: 10, height: 160, format: move |v: f64| format!("{v:.0}") }
        }
    }
}
