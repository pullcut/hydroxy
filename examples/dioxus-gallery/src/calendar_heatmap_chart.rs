use dioxus::prelude::*;
use oxide_dioxus::calendar_heatmap_chart::CalendarHeatmapChart;
use oxide_dioxus::chart_frame::ChartFrame;

#[component]
pub fn CalendarHeatmapChartDemo() -> Element {
    // A year of commits from 29 Oct 2025: busy weekdays, quiet weekends, a two-week break at Christmas.
    let mut next = noise(7);
    let commits: Vec<f64> = (0..365)
        .map(|d: u32| {
            let weekend = (d + 2) % 7 >= 5;
            let holiday = (57..71).contains(&d);
            let busy = if weekend || holiday { 0.6 } else { 6.0 };
            (busy * next() * 2.0).floor()
        })
        .collect();
    // Two years of daily rainfall: wet winters, dry summers, many dry days.
    let mut next = noise(23);
    let rain: Vec<f64> = (0..731)
        .map(|d: u32| {
            let winter = 1.0 + 0.6 * (f64::from(d) / 365.25 * std::f64::consts::TAU).cos();
            if next() < 0.45 { 0.0 } else { (winter * 9.0 * next().powi(2) * 10.0).round() / 10.0 }
        })
        .collect();

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame {
                title: "Commits",
                description: "Commits per day over the last year. Hover a day, or focus the calendar and use the arrow keys.",
                CalendarHeatmapChart { values: commits, start: (2025, 10, 29), label: "Commits per day, 29 Oct 2025 to 28 Oct 2026" }
            }
            ChartFrame {
                title: "Daily rainfall",
                description: "Millimetres per day. Spans over a year stack one row per calendar year.",
                export: true,
                CalendarHeatmapChart {
                    values: rain,
                    start: (2024, 1, 1),
                    label: "Daily rainfall in millimetres, 2024 and 2025",
                    format: move |v: f64| format!("{v:.1} mm"),
                }
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
