use dioxus::prelude::*;
use oxide_dioxus::renko_chart::RenkoChart;

#[component]
pub fn RenkoChartDemo() -> Element {
    let closes = vec![
        212.34, 214.99, 213.28, 215.41, 215.04, 214.67, 218.62, 219.08, 219.15, 220.76, 223.16, 223.25, 224.58, 222.78,
        222.19, 221.47, 218.95, 216.09, 212.98, 212.66, 212.46, 211.97, 212.26, 209.74, 209.73, 210.35, 212.01, 210.46,
        209.81, 205.93, 205.08, 200.83, 198.14, 200.50, 196.24, 197.99, 198.80, 198.32, 199.39, 200.60,
    ];

    rsx! {
        RenkoChart { prices: closes, label: "ACME closes, brick size $2", brick_size: 2.0, format: move |v: f64| format!("${v:.0}") }
    }
}
