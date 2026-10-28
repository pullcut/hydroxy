use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::heatmap_chart::HeatmapChart;

#[component]
pub fn HeatmapChartDemo() -> Element {
    let days: Vec<String> = ["Mon", "Tue", "Wed", "Thu", "Fri"].map(String::from).to_vec();
    let hours: Vec<String> = ["9", "10", "11", "12", "13", "14", "15", "16"].map(String::from).to_vec();
    let sessions = vec![
        vec![12.0, 24.0, 31.0, 18.0, 9.0, 22.0, 27.0, 14.0],
        vec![15.0, 29.0, 36.0, 21.0, 11.0, 26.0, 33.0, 17.0],
        vec![14.0, 27.0, 40.0, 25.0, 13.0, 30.0, 35.0, 19.0],
        vec![11.0, 23.0, 33.0, 19.0, f64::NAN, 24.0, 29.0, 13.0],
        vec![8.0, 16.0, 21.0, 12.0, 6.0, 10.0, 12.0, 5.0],
    ];

    // A year of hourly load: 8,760 cells, so the grid draws on a canvas with sparse labels.
    let mut next = noise(11);
    let hours_of_day: Vec<String> = (0..24).map(|h| format!("{h:02}:00")).collect();
    let month_days = [
        ("Jan", 31),
        ("Feb", 28),
        ("Mar", 31),
        ("Apr", 30),
        ("May", 31),
        ("Jun", 30),
        ("Jul", 31),
        ("Aug", 31),
        ("Sep", 30),
        ("Oct", 31),
        ("Nov", 30),
        ("Dec", 31),
    ];
    let dates: Vec<String> = month_days.iter().flat_map(|&(m, n)| (1..=n).map(move |d| format!("{d} {m}"))).collect();
    let load: Vec<Vec<f64>> = (0..24)
        .map(|h| {
            let daily = (-(f64::from(h) - 14.0).powi(2) / 18.0).exp();
            (0..365)
                .map(|d| {
                    let season = 1.0 + 0.3 * (f64::from(d) / 365.0 * std::f64::consts::TAU).cos();
                    let weekend = if (d + 3) % 7 >= 5 { 0.55 } else { 1.0 };
                    (20.0 + 80.0 * daily * season * weekend * (1.0 + 0.15 * normal(&mut next))).round()
                })
                .collect()
        })
        .collect();

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame {
                title: "Sessions by weekday and hour",
                description: "Darker cells are busier. Hover a cell, or focus the grid and use the arrow keys.",
                HeatmapChart {
                    values: sessions,
                    rows: days,
                    columns: hours,
                    label: "Sessions by weekday and hour of day",
                    show_values: true,
                }
            }
            ChartFrame {
                title: "Server load through 2026",
                description: "Requests per second, by hour of day. 8,760 cells drawn on a canvas.",
                export: true,
                HeatmapChart {
                    values: load,
                    rows: hours_of_day,
                    columns: dates,
                    label: "Requests per second by hour and day",
                    cell_height: 10,
                }
            }
        }
    }
}

/// A small deterministic random source (xorshift), so every platform draws the same data.
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
