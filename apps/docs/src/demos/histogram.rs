use leptos::prelude::*;
use oxide_leptos::histogram::Histogram;

pub fn demo() -> AnyView {
    // A right-skewed spread of response times: most are quick, a few are slow.
    let times: Vec<f64> = (0..160)
        .map(|i| {
            let t = (f64::from(i) * 0.618_034).fract();
            60.0 + 340.0 * t * t * t + f64::from(i % 7) * 4.0
        })
        .collect();

    view! {
        <div style="display: grid; gap: var(--space-10);">
            // Sturges' rule picks the bin count.
            <Histogram values=times.clone() label="Response time" format=|v: f64| format!("{v:.0} ms") />

            // Or set it yourself.
            <Histogram values=times label="Response time, 10 bins" bins=10 height=160 format=|v: f64| format!("{v:.0}") />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "histogram",
    name: "Histogram",
    group: "Charts",
    summary: "Show how a set of values is distributed. Values are binned into equal-width bars by Sturges' rule, or into a bin count you choose. Hover a bar for its count.",
    demo,
    leptos: include_str!("histogram.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/histogram.rs"),
    parts: &[
        &["Histogram", "root", "figure with a screen-reader table of every bin"],
        &["", "gridline", "Recessive count gridlines at nice ticks"],
        &["", "column", "One per bin; its title reads the range and count"],
        &["", "bar", "Bar in the first chart colour, growing from the baseline"],
        &["", "label", "Bin edges along the bottom, thinned to fit"],
    ],
    keys: &[],
    props: &[
        &["values", "Vec<f64>", "Required. Raw observations; non-finite values are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["bins", "usize", "Number of bins. Default: Sturges' rule, ceil(log2 n) + 1"],
        &["height", "u16", "Plot height in px. Default 200"],
        &["format", "Fn(f64) -> String", "Formats the bin edges"],
    ],
};
