use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::spiral_chart::SpiralChart;

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

#[component]
pub fn SpiralChartDemo() -> Element {
    // Three years of daily cycling, one turn per year: long weekend rides in summer,
    // more rest days in winter. 2023 starts on a Sunday.
    let mut dates = Vec::new();
    for year in 2023..=2025 {
        let leap = year % 4 == 0;
        for (m, days) in [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31].into_iter().enumerate() {
            dates.extend((1..=days).map(|d| format!("{d} {} {year}", MONTHS[m])));
        }
    }
    let mut next = noise(41);
    let distance: Vec<f64> = (0..dates.len())
        .map(|d| {
            #[allow(clippy::cast_precision_loss, reason = "day counts are small")]
            let summer = 0.5 - 0.5 * (d as f64 / 365.25 * std::f64::consts::TAU).cos();
            let weekend = matches!(d % 7, 0 | 6);
            if next() > 0.45 + 0.4 * summer {
                0.0
            } else if weekend {
                (35.0 + 55.0 * summer * next()).round()
            } else {
                (8.0 + 18.0 * summer + 6.0 * next()).round()
            }
        })
        .collect();
    let spokes: Vec<String> = MONTHS.map(String::from).to_vec();
    let turns: Vec<String> = ["2023", "2024", "2025"].map(String::from).to_vec();

    rsx! {
        ChartFrame {
            title: "Daily cycling distance",
            description: "Kilometres a day, one turn per year from the centre out. Summers bloom as darker arcs along the same months.",
            export: true,
            SpiralChart {
                values: distance,
                labels: dates,
                period: 365.25,
                spokes,
                turns,
                label: "Daily cycling distance in kilometres, 2023 to 2025",
                format: move |v: f64| format!("{v:.0} km"),
            }
        }
    }
}

/// A small deterministic random source (xorshift), so every render draws the same data.
#[allow(clippy::cast_precision_loss, reason = "a 53-bit fraction is exact")]
fn noise(seed: u64) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    }
}
