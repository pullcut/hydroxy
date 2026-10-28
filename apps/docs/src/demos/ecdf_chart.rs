use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::ecdf_chart::*;
use leptos::prelude::*;

/// Page loads in seconds around `median`, skewed right like real load times.
fn loads(median: f64, spread: f64, seed: u32) -> Vec<f64> {
    let mut s = seed;
    let mut next = move || {
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(s >> 8) / f64::from(1u32 << 24)
    };
    (0..240).map(|_| median * (spread * (next() + next() + next() + next() - 2.0) * 1.7).exp()).collect()
}

pub fn demo() -> AnyView {
    let regions = vec![
        Sample::new("Europe", loads(1.4, 0.28, 7)),
        Sample::new("Americas", loads(1.7, 0.32, 11)),
        Sample::new("Asia Pacific", loads(2.3, 0.4, 23)),
    ];

    view! {
        <ChartFrame title="Page load time by region" description="Share of page loads finished within each time. Dashed lines are normal fits.">
            <EcdfChart
                data=regions
                label="Cumulative distribution of page load time by region"
                reference=true
                format=|v: f64| format!("{v:.1} s")
                annotations=vec![Annotation::x_line(3.0, "3 s budget")]
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "ecdf-chart",
    name: "ECDF chart",
    group: "Distributions",
    summary: "Compare whole distributions without binning. Each group's line climbs by one step per value, so the height at any x is the share at or below it and percentiles read straight off.",
    demo,
    leptos: include_str!("ecdf_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/ecdf_chart.rs"),
    parts: &[
        &["EcdfChart", "root", "figure with a screen-reader table of each group's size, mean, SD and percentiles"],
        &["", "line", "One step line per group, drawn left to right"],
        &["", "reference", "Each group's fitted normal CDF, dashed (reference)"],
        &["", "crosshair", "The hovered value, with a dot on each line"],
        &["", "tooltip", "Each group's percentile at the hovered value"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next value"],
        &["Home / End", "Smallest or largest value"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Sample>", "Required. Sample::new(name, values). Up to six groups"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["reference", "bool", "Dashed normal fit per group"],
        &["height", "u16", "Plot height in px. Default 260"],
        &["format", "Fn(f64) -> String", "Formats values"],
        &["annotations", "Vec<Annotation>", "x is a value, y a share from 0 to 1"],
    ],
};
