use leptos::prelude::*;
use oxide_leptos::calendar_heatmap_chart::CalendarHeatmapChart;
use oxide_leptos::chart_frame::ChartFrame;

pub fn demo() -> AnyView {
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

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Commits" description="Commits per day over the last year. Hover a day, or focus the calendar and use the arrow keys.">
                <CalendarHeatmapChart values=commits start=(2025, 10, 29) label="Commits per day, 29 Oct 2025 to 28 Oct 2026" />
            </ChartFrame>
            <ChartFrame title="Daily rainfall" description="Millimetres per day. Spans over a year stack one row per calendar year." export=true>
                <CalendarHeatmapChart
                    values=rain
                    start=(2024, 1, 1)
                    label="Daily rainfall in millimetres, 2024 and 2025"
                    format=|v: f64| format!("{v:.1} mm")
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

/// A small deterministic random source (xorshift), so server and browser draw the same data.
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

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "calendar-heatmap-chart",
    name: "Calendar heatmap chart",
    group: "Time",
    summary: "One cell per day in week columns, shaded by value, like a contribution graph. Weekly rhythms read down the columns and seasons across them. Spans over a year stack one row per calendar year so the years line up.",
    demo,
    leptos: include_str!("calendar_heatmap_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/calendar_heatmap_chart.rs"),
    parts: &[
        &["CalendarHeatmapChart", "root", "figure with a screen-reader table of every day"],
        &["", "block", "One grid of week columns: the whole span, or one calendar year"],
        &["", "year", "The year above each block when the span is split"],
        &["", "month", "Month names over the week each month starts in"],
        &["", "weekday", "Mon, Wed and Fri down the left"],
        &["", "cell", "One per day, shaded from zero (or the lowest value) to the highest"],
        &["", "tooltip", "The date and value, above the day"],
        &["", "scale", "The colour ramp with its lowest and highest values"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Previous or next day"],
        &["ArrowLeft / ArrowRight", "Same weekday a week earlier or later"],
        &["Home / End", "First or last day"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["values", "Vec<f64>", "Required. One value per day from start; NaN is a day with no data"],
        &["start", "(i64, u32, u32)", "Required. The first day as (year, month, day)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["format", "Fn(f64) -> String", "Formats the tooltip, scale and table"],
    ],
};
