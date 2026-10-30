use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::parallel_coordinates_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let mut next = noise(7);
    // Sixty laptops in three classes.
    let classes = [
        ("Ultrabook", 1_250.0, 1.2, 14.0, 6_800.0),
        ("Gaming", 1_900.0, 2.4, 6.0, 14_500.0),
        ("Workstation", 2_600.0, 2.0, 9.0, 11_200.0),
    ];
    let mut laptops: [Vec<f64>; 4] = Default::default();
    let mut classes_of = Vec::new();
    let mut names = Vec::new();
    for i in 0..60 {
        let (class, price, weight, battery, score) = classes[i % 3];
        let tier = next();
        laptops[0].push((price * (0.7 + 0.7 * tier) / 10.0).round() * 10.0);
        laptops[1].push(((weight * (0.85 + 0.3 * next())) * 100.0).round() / 100.0);
        laptops[2].push((battery * (0.8 + 0.4 * next())).round());
        laptops[3].push((score * (0.75 + 0.5 * tier)).round());
        classes_of.push(class.to_string());
        names.push(format!("{class} {}", 100 + i * 7));
    }
    let [price, weight, battery, score] = laptops;
    let dims = vec![
        Dimension::new("Price (£)", price),
        Dimension::new("Weight (kg)", weight),
        Dimension::new("Battery (h)", battery),
        Dimension::new("Benchmark", score),
    ];

    // Four thousand API requests: too many for one element each, so they draw on a canvas.
    let endpoints = ["/search", "/checkout", "/profile"];
    let mut requests: [Vec<f64>; 4] = Default::default();
    let mut endpoint_of = Vec::new();
    for i in 0..4_000 {
        let e = i % 3;
        let size = [24.0, 8.0, 3.0][e] * (0.5 + next());
        let cpu = [38.0, 22.0, 9.0][e] * (0.6 + 0.8 * next());
        let latency = 20.0 + cpu * 2.4 + size * 1.5 + 40.0 * next().powi(3);
        requests[0].push(latency.round());
        requests[1].push((size * 10.0).round() / 10.0);
        requests[2].push(cpu.round());
        requests[3].push((60.0 + size * 4.0 + 30.0 * next()).round());
        endpoint_of.push(endpoints[e].to_string());
    }
    let [latency, payload, cpu, memory] = requests;
    let api = vec![
        Dimension::new("Latency (ms)", latency),
        Dimension::new("Payload (KB)", payload),
        Dimension::new("CPU (ms)", cpu),
        Dimension::new("Memory (MB)", memory),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Laptops by price, weight, battery and speed" description="Drag along an axis to filter; brushes on several axes combine. Click an axis to clear its brush.">
                <ParallelCoordinatesChart dimensions=dims groups=classes_of names=names label="Laptops by price, weight, battery life and benchmark score" />
            </ChartFrame>
            <ChartFrame title="API requests" description="4,000 requests in the last minute, drawn on a canvas." export=true>
                <ParallelCoordinatesChart dimensions=api groups=endpoint_of label="API requests by latency, payload, CPU time and memory" height=280 />
            </ChartFrame>
        </div>
    }
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
    slug: "parallel-coordinates-chart",
    name: "Parallel coordinates",
    group: "Points",
    summary: "One vertical axis per measure, each on its own scale, and one line per row crossing them, so patterns across many measures show as bundles. Drag along an axis to keep a range; brushes on several axes combine and everything else dims. Colour picks out a category. Past 2,000 rows the lines draw on a canvas, so thousands stay fast.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("parallel_coordinates_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/parallel_coordinates_chart.rs"),
    parts: &[
        &["ParallelCoordinatesChart", "root", "figure; data-filtered while any brush is set"],
        &["", "lines", "One path per row, revealed left to right"],
        &["", "line", "A row in its category colour; data-dim when filtered out"],
        &["", "canvas", "Draws the lines instead past 2,000 rows"],
        &["", "axis", "A dimension's axis line, with its name and ticks"],
        &["", "brush", "The kept range on an axis"],
        &["", "overlay", "Focusable; drag on an axis to brush, hover a line to read it"],
        &["", "active-line", "The hovered or focused row, drawn on top"],
        &["", "tooltip", "The row's name, category and every value"],
        &["", "count", "Rows kept, out of all of them"],
        &["", "clear", "Removes every brush"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next row that passes the filters"],
        &["Home / End", "First or last row that passes"],
        &["Escape", "Clear the active row"],
    ],
    props: &[
        &[
            "dimensions",
            "Vec<Dimension>",
            "Required. Dimension::new(name, values), one value per row; NaN leaves a gap",
        ],
        &["groups", "Vec<String>", "Each row's category, which picks its colour. Up to six"],
        &["names", "Vec<String>", "Each row's name for the tooltip and table. Default \"Row n\""],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 320"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
    ],
    events: crate::doc::FILTER_EVENTS,
};
