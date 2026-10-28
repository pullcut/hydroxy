use dioxus::prelude::*;
use hydroxy_dioxus::bar_race_chart::{BarRaceChart, Series};
use hydroxy_dioxus::chart_frame::ChartFrame;

#[component]
pub fn BarRaceChartDemo() -> Element {
    let quarters: Vec<String> = (2022..=2024).flat_map(|y| (1..=4).map(move |q| format!("Q{q} {y}"))).collect();
    // Monthly active users in thousands: each city starts somewhere and grows at its own pace.
    let grow = |start: f64, rate: f64| (0..12).map(|q| (start * rate.powi(q)).round()).collect::<Vec<f64>>();
    let cities = vec![
        Series::new("London", grow(182.0, 1.04)),
        Series::new("Berlin", grow(121.0, 1.09)),
        Series::new("Paris", grow(140.0, 1.06)),
        Series::new("Madrid", grow(64.0, 1.12)),
        Series::new("Amsterdam", grow(88.0, 1.07)),
        Series::new("Lisbon", grow(22.0, 1.24)),
        Series::new("Dublin", grow(51.0, 1.08)),
        Series::new("Stockholm", grow(47.0, 1.10)),
        Series::new("Warsaw", grow(18.0, 1.26)),
        Series::new("Milan", grow(70.0, 1.05)),
        Series::new("Prague", grow(29.0, 1.15)),
        Series::new("Vienna", grow(39.0, 1.09)),
    ];

    rsx! {
        ChartFrame { title: "Monthly active users by city", description: "The top eight cities each quarter, in thousands. Press play, or drag to scrub.",
            BarRaceChart {
                frames: quarters,
                series: cities,
                label: "Monthly active users by city, 2022 to 2024",
                top: 8,
                format: move |v: f64| format!("{v:.0}k"),
            }
        }
    }
}
