use hydroxy_leptos::bar_race_chart::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use leptos::prelude::*;

pub fn demo() -> AnyView {
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

    view! {
        <ChartFrame title="Monthly active users by city" description="The top eight cities each quarter, in thousands. Press play, or drag to scrub.">
            <BarRaceChart frames=quarters series=cities label="Monthly active users by city, 2022 to 2024" top=8 format=|v: f64| format!("{v:.0}k") />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bar-race-chart",
    name: "Bar race chart",
    group: "Bars",
    summary: "A ranking played through time: the top entries as bars that grow and glide past each other as the order changes, with play, pause and a scrubber.",
    demo,
    leptos: include_str!("bar_race_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bar_race_chart.rs"),
    parts: &[
        &[
            "BarRaceChart",
            "root",
            "figure with a screen-reader table of every value in every frame; data-playing while it runs",
        ],
        &["", "row", "One entry: name, bar and value, placed by rank; data-out when outside the top"],
        &["", "bar", "Bar length, as a share of the leader's value"],
        &["", "value", "The value, counting as the race plays"],
        &["", "frame", "The current frame, large in the corner"],
        &["", "play", "Play or pause button"],
        &["", "scrubber", "Range input over the frames"],
    ],
    keys: &[
        &["Space / Enter", "Play or pause (on the button)"],
        &["ArrowLeft / ArrowRight", "Previous or next frame (on the scrubber)"],
        &["Home / End", "First or last frame (on the scrubber)"],
    ],
    props: &[
        &["frames", "Vec<String>", "Required. One label per frame"],
        &["series", "Vec<Series>", "Required. Series::new(name, one value per frame)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["top", "usize", "Bars shown at once. Default 10"],
        &["frame_ms", "u32", "How long each frame lasts while playing. Default 1200"],
        &["autoplay", "bool", "Start once mounted; never under reduced motion"],
        &["row_height", "u16", "Row height in px. Default 30"],
        &["format", "Fn(f64) -> String", "Formats values, ticks and table"],
    ],
};
