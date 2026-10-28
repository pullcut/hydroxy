use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::horizon_chart::{HorizonChart, Series};
use leptos::prelude::*;

pub fn demo() -> AnyView {
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

    view! {
        <ChartFrame
            title="Temperature anomaly by city"
            description="Hourly difference from the 1991–2020 normal, through 2025. Blue is warmer, red is colder; darker shades are further from normal."
            export=true
        >
            <HorizonChart
                series
                times
                label="Hourly temperature anomaly by city, 2025"
                format=|v: f64| format!("{v:+.1} °C")
            />
        </ChartFrame>
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

/// A standard normal sample from two uniform ones (Box-Muller).
fn normal(next: &mut impl FnMut() -> f64) -> f64 {
    (-2.0 * next().max(1e-12).ln()).sqrt() * (std::f64::consts::TAU * next()).cos()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "horizon-chart",
    name: "Horizon chart",
    group: "Lines and areas",
    summary: "Many time series stacked as thin rows. Each row's area is cut into equal bands that fold down on top of each other, so a tall line fits a short row and darker shades mean bigger values; values below zero fold up in a second hue. Every row shares one band size, so rows compare.",
    demo,
    leptos: include_str!("horizon_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/horizon_chart.rs"),
    parts: &[
        &["HorizonChart", "root", "figure with a screen-reader table of every reading"],
        &["", "row", "One per series: an SVG of folded bands"],
        &["", "fold", "One band's area, shaded by how far above or below zero it sits"],
        &["", "x-label", "Series names, down the left"],
        &["", "tick", "Calendar ticks along the shared time axis"],
        &["", "crosshair", "The moment under the pointer, across every row"],
        &["", "tooltip", "The time and every series' value, the row under the pointer in bold"],
        &["", "legend", "The value range each shade covers"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next reading"],
        &["ArrowUp / ArrowDown", "Previous or next row"],
        &["Home / End", "First or last reading"],
        &["Escape", "Hide the crosshair"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. One row each, top to bottom; NaN leaves a gap"],
        &["times", "Vec<f64>", "Required. UTC seconds since the epoch, ascending, shared by every series"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["bands", "usize", "Colour bands per row, 2 to 4. Default 3"],
        &["row_height", "u16", "Row height in px. Default 36"],
        &["format", "Fn(f64) -> String", "Formats the legend, tooltip and table"],
    ],
};
