use dioxus::prelude::*;
use oxide_dioxus::contour_chart::{ContourChart, HeatField};

/// Two overlapping hills on a 36 × 20 grid.
fn terrain() -> Vec<f64> {
    let hill = |x: f64, y: f64, cx: f64, cy: f64, h: f64, r: f64| h * (-((x - cx).powi(2) + (y - cy).powi(2)) / (2.0 * r * r)).exp();
    (0..20)
        .flat_map(|r| (0..36).map(move |c| (f64::from(c), f64::from(r))))
        .map(|(x, y)| hill(x, y, 11.0, 7.0, 80.0, 5.0) + hill(x, y, 25.0, 13.0, 60.0, 6.0))
        .collect()
}

#[component]
pub fn ContourChartDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ContourChart {
                values: terrain(),
                columns: 36,
                label: "Elevation in metres",
                filled: true,
                format: move |v: f64| format!("{v:.0} m"),
            }
            ContourChart { values: terrain(), columns: 36, label: "Elevation contours", levels: 8 }
            HeatField { values: terrain(), columns: 36, label: "Elevation heat field" }
        }
    }
}
