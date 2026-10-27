use leptos::prelude::*;
use oxide_leptos::renko_chart::RenkoChart;

pub fn demo() -> AnyView {
    let closes = vec![
        212.34, 214.99, 213.28, 215.41, 215.04, 214.67, 218.62, 219.08, 219.15, 220.76, 223.16, 223.25, 224.58, 222.78,
        222.19, 221.47, 218.95, 216.09, 212.98, 212.66, 212.46, 211.97, 212.26, 209.74, 209.73, 210.35, 212.01, 210.46,
        209.81, 205.93, 205.08, 200.83, 198.14, 200.50, 196.24, 197.99, 198.80, 198.32, 199.39, 200.60,
    ];

    view! { <RenkoChart prices=closes label="ACME closes, brick size $2" brick_size=2.0 format=|v: f64| format!("${v:.0}") /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "renko-chart",
    name: "Renko chart",
    group: "Charts",
    summary: "Fixed-size bricks laid each time price moves a full brick from the last one, ignoring time. Small moves are filtered out, so trends and reversals stand out.",
    demo,
    leptos: include_str!("renko_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/renko_chart.rs"),
    parts: &[
        &["RenkoChart", "root", "figure with a screen-reader table of every brick"],
        &["", "overlay", "Focusable plot; arrow keys move between bricks"],
        &["", "brick", "One per brick, data-dir up or down"],
        &["", "tooltip", "Open and close of the active brick, and the price that completed it"],
        &["", "annotation", "Reference line, band or point"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous brick"],
        &["Home / End", "First or last brick"],
        &["Escape", "Clear"],
    ],
    props: &[
        &["prices", "Vec<f64>", "Required. Prices in time order, such as daily closes"],
        &["labels", "Vec<String>", "One per price, such as a date, named in the tooltip"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["brick_size", "f64", "Brick size in price units. Default: a twentieth of the range, rounded"],
        &["annotations", "Vec<Annotation>", "Y in prices, x in brick indices"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats prices on the axis, tooltip and table"],
    ],
};
