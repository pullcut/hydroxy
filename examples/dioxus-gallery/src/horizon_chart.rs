use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::horizon_chart::{HorizonChart, Series};

#[component]
pub fn HorizonChartDemo() -> Element {
    // Hourly temperature anomaly through 2025 for six cities: 8,760 readings each, thinned
    // to the screen with peaks kept. Each city drifts warm or cold for days at a time.
    let start = 1_735_689_600.0; // 1 Jan 2025, 00:00 UTC
    let times: Vec<f64> = (0..8_760).map(|h| start + f64::from(h) * 3_600.0).collect();
    let cities = ["Oslo", "Berlin", "Madrid", "Cairo", "Mumbai", "Tokyo"];
    let series: Vec<Series> = cities
        .iter()
        .zip(1u64..)
        .map(|(name, seed)| {
            let mut next = noise(seed * 97);
            let mut drift = 0.0;
            let values: Vec<f64> = (0..8_760)
                .map(|h| {
                    drift = drift * 0.995 + 0.16 * normal(&mut next);
                    let daily = 0.6 * (f64::from(h % 24) / 24.0 * std::f64::consts::TAU).sin() * next();
                    ((drift + daily) * 10.0).round() / 10.0
                })
                .collect();
            Series::new(*name, values)
        })
        .collect();

    rsx! {
        ChartFrame {
            title: "Temperature anomaly by city",
            description: "Hourly difference from the 1991–2020 normal, through 2025. Blue is warmer, red is colder; darker shades are further from normal.",
            export: true,
            HorizonChart {
                series,
                times,
                label: "Hourly temperature anomaly by city, 2025",
                format: move |v: f64| format!("{v:+.1} °C"),
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

/// A standard normal sample from two uniform ones (Box-Muller).
fn normal(next: &mut impl FnMut() -> f64) -> f64 {
    (-2.0 * next().max(1e-12).ln()).sqrt() * (std::f64::consts::TAU * next()).cos()
}
