use dioxus::prelude::*;
use hydroxy_dioxus::smith_chart::{Sample, SmithChart};

#[component]
pub fn SmithChartDemo() -> Element {
    let samples = vec![
        Sample::new("Antenna", 0.35, -0.60).labelled("1 GHz"),
        Sample::new("Antenna", 0.60, -0.30),
        Sample::new("Antenna", 1.00, 0.10),
        Sample::new("Antenna", 1.80, 0.70),
        Sample::new("Antenna", 3.00, 1.50).labelled("3 GHz"),
        Sample::new("Filter", 0.50, 1.40).labelled("1 GHz"),
        Sample::new("Filter", 0.90, 0.80),
        Sample::new("Filter", 1.30, -0.20),
        Sample::new("Filter", 0.70, -1.10).labelled("3 GHz"),
    ];

    rsx! {
        SmithChart { samples, label: "Input impedance, 1 to 3 GHz" }
    }
}
