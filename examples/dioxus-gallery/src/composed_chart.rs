use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::composed_chart::{Annotation, ComposedChart, Curve, Layer};

#[component]
pub fn ComposedChartDemo() -> Element {
    let months: Vec<String> = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].map(String::from).to_vec();
    // Orders and returns as bars, the rolling average as a line, and promotions as points.
    let layers = vec![
        Layer::bar("Orders", [312.0, 288.0, 341.0, 365.0, 402.0, 389.0, 420.0, 455.0, 431.0, 470.0, 512.0, 598.0]),
        Layer::bar("Returns", [-24.0, -19.0, -28.0, -31.0, -26.0, -35.0, -30.0, -38.0, -33.0, -41.0, -45.0, -62.0]),
        Layer::line("3-month average", [f64::NAN, f64::NAN, 314.0, 331.0, 369.0, 385.0, 404.0, 421.0, 435.0, 452.0, 471.0, 527.0]),
        Layer::points("Promotion weeks", [f64::NAN, f64::NAN, 341.0, f64::NAN, f64::NAN, f64::NAN, 420.0, f64::NAN, f64::NAN, f64::NAN, 512.0, 598.0]),
    ];
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].map(String::from).to_vec();
    let traffic = vec![Layer::area("Visits (k)", [182.0, 214.0, 236.0, 271.0]), Layer::bar("Signups", [96.0, 121.0, 118.0, 154.0])];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame {
                title: "Orders",
                description: "Monthly orders and returns, with the rolling average and promotion months.",
                export: true,
                ComposedChart {
                    layers,
                    labels: months,
                    label: "Orders, returns and the rolling average by month",
                    curve: Curve::Monotone,
                    annotations: vec![Annotation::y_line(450.0, "Target"), Annotation::x_band(10.0, 11.0, "Peak season")],
                }
            }
            ChartFrame { title: "Traffic and signups", description: "An area and bars on one axis.",
                ComposedChart { layers: traffic, labels: quarters, label: "Visits and signups by quarter", height: 200 }
            }
        }
    }
}
