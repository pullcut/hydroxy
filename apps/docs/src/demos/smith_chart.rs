use hydroxy_leptos::smith_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // Normalised impedance (z / Z0) swept across frequency. The first and last labels name the ends of each trace.
    let samples = vec![
        Sample::new("Antenna", 0.35, -0.60).labelled("1 GHz"),
        Sample::new("Antenna", 0.60, -0.30),
        Sample::new("Antenna", 1.00, 0.10),
        Sample::new("Antenna", 1.80, 0.70),
        Sample::new("Antenna", 3.00, 1.50).labelled("3 GHz"),
        Sample::new("Filter", 0.50, 1.40).labelled("1 GHz"),
        Sample::new("Filter", 0.90, 0.80),
        Sample::new("Filter", 1.30, -0.20),
        Sample::new("Filter", 0.70, -1.10).labelled("3 GHz"),
    ];

    view! {
        <SmithChart samples=samples label="Input impedance, 1 to 3 GHz" />
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "smith-chart",
    name: "Smith chart",
    group: "Points",
    summary: "Plot impedance sweeps as reflection coefficients inside the unit disc, with the constant-resistance and constant-reactance grid. For RF work. Hover, touch or step through samples with the keys to read each impedance and |Γ|; hovering a legend row picks out its trace.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("smith_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/smith_chart.rs"),
    parts: &[
        &["SmithChart", "root", "figure; `data-highlight` while a trace is picked out"],
        &["", "plot", "Focusable group; reads the sample nearest the pointer"],
        &["", "boundary", "The unit circle: zero resistance"],
        &["", "grid", "Constant-resistance circles and constant-reactance arcs"],
        &["", "series", "One group per trace; `data-active` when picked out, the others quiet"],
        &["", "point", "One per sample; the sweep ends are filled; `data-active` when read"],
        &["", "end-label", "Labels at the start and end of a sweep"],
        &["", "tooltip", "Trace, sample label, impedance and |Γ|"],
        &["", "legend", "One row per trace when there are two or more; hover picks the trace out"],
        &["", "sr-live", "Announces the sample being read"],
        &["", "sr-table", "Every sample, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous sample along the trace"],
        &["ArrowDown / ArrowUp", "Same position on the next or previous trace"],
        &["Home / End", "First or last sample of the trace"],
        &["Escape", "Clear the tooltip"],
    ],
    props: &[
        &[
            "samples",
            "Vec<Sample>",
            "Required. Sample::new(trace, real, imaginary).labelled(..); same trace name is one sweep",
        ],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["z0", "f64", "Reference impedance. 1 for normalised samples (default); 50 for raw ohms"],
        &["format", "Fn(f64) -> String", "Formats resistance and reactance. Default: two decimal places"],
    ],
    events: crate::doc::CHART_EVENTS,
};
