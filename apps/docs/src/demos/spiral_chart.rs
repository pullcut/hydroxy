use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::spiral_chart::SpiralChart;

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

pub fn demo() -> AnyView {
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

    view! {
        <ChartFrame
            title="Daily cycling distance"
            description="Kilometres a day, one turn per year from the centre out. Summers bloom as darker arcs along the same months."
            export=true
        >
            <SpiralChart
                values=distance
                labels=dates
                period=365.25
                spokes
                turns
                label="Daily cycling distance in kilometres, 2023 to 2025"
                format=|v: f64| format!("{v:.0} km")
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

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "spiral-chart",
    name: "Spiral chart",
    group: "Radial",
    summary: "A time series wound onto a spiral, one turn per period, so the same moment in every period lines up along a spoke. Each value is a short arc shaded by value; recurring patterns show as wedges of colour. The spiral draws out from the centre.",
    demo,
    leptos: include_str!("spiral_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/spiral_chart.rs"),
    parts: &[
        &["SpiralChart", "root", "figure with a screen-reader table of every value"],
        &["", "segment", "One arc per value, shaded from the lowest to the highest"],
        &["", "spoke", "A faint line through every turn at each dial name"],
        &["", "spoke-label", "Dial names around the outside, such as months"],
        &["", "turn-label", "Each turn's name at 12 o'clock, such as the year"],
        &["", "highlight", "The arc under the pointer or keyboard focus"],
        &["", "tooltip", "The value's name and value"],
        &["", "scale", "The colour ramp with its lowest and highest values"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next value along the spiral"],
        &["ArrowUp / ArrowDown", "One turn out or in"],
        &["Home / End", "First or last value"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["values", "Vec<f64>", "Required. Values in time order; NaN leaves a gap"],
        &["labels", "Vec<String>", "Required. A name per value, such as its date"],
        &["period", "f64", "Required. Values per turn: 365.25 for daily values over years"],
        &["spokes", "Vec<String>", "Names spaced evenly around the dial from 12 o'clock"],
        &["turns", "Vec<String>", "A name per turn, from the centre out"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Diameter in px. Default 360"],
        &["format", "Fn(f64) -> String", "Formats the tooltip, scale and table"],
    ],
};
