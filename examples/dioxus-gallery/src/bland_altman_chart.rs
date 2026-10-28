use dioxus::prelude::*;
use hydroxy_dioxus::bland_altman_chart::BlandAltmanChart;
use hydroxy_dioxus::chart_frame::ChartFrame;

#[component]
pub fn BlandAltmanChartDemo() -> Element {
    // Blood glucose in mmol/L for 40 patients: the clinic's lab analyser and a finger-prick
    // meter. The meter reads a little high, and more so at high readings.
    let lab: Vec<f64> = (0..40).map(|i| 4.2 + f64::from((i * 37) % 97) / 10.0).collect();
    let meter: Vec<f64> =
        lab.iter().zip(0..).map(|(v, i)| v * 1.03 + 0.15 + f64::from((i * 53) % 17) / 20.0 - 0.4).collect();

    rsx! {
        ChartFrame { title: "Glucose meter against the lab", description: "Each patient's two readings: their mean along the bottom, meter minus lab up the side.",
            BlandAltmanChart {
                a: meter,
                b: lab,
                a_label: "Meter",
                b_label: "Lab",
                label: "Bland-Altman plot of meter and lab glucose",
                show_ci: true,
                format: move |v: f64| format!("{v:.1}"),
            }
        }
    }
}
