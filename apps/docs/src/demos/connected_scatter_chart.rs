use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::connected_scatter_chart::*;

pub fn demo() -> AnyView {
    // Annual averages: unemployment rate and consumer price inflation, both in percent.
    let years = [
        (2008, 5.7, 3.6), (2009, 7.6, 2.2), (2010, 7.9, 3.3), (2011, 8.1, 4.5), (2012, 8.0, 2.8), (2013, 7.6, 2.6),
        (2014, 6.2, 1.5), (2015, 5.4, 0.0), (2016, 4.9, 0.7), (2017, 4.4, 2.7), (2018, 4.1, 2.5), (2019, 3.8, 1.8),
        (2020, 4.6, 0.9), (2021, 4.5, 2.6), (2022, 3.9, 9.1), (2023, 4.0, 7.3), (2024, 4.3, 2.5),
    ];
    let data: Vec<Moment> = years.iter().map(|&(y, u, i)| Moment::new(y.to_string(), u, i)).collect();

    view! {
        <ChartFrame title="Unemployment and inflation" description="Each dot is one year, joined in order from 2008 (hollow) to 2024 (solid).">
            <ConnectedScatterChart
                data
                label="Unemployment against inflation, 2008 to 2024"
                x_label="Unemployment rate"
                y_label="Inflation"
                format_x=|v: f64| format!("{v:.1}%")
                format_y=|v: f64| format!("{v:.1}%")
                annotations=vec![Annotation::y_line(2.0, "2% target")]
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "connected-scatter-chart",
    name: "Connected scatter chart",
    group: "Points",
    summary: "Two measures plotted against each other and joined in time order, so the path shows how they moved together. The line draws itself from the first moment to the last.",
    demo,
    leptos: include_str!("connected_scatter_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/connected_scatter_chart.rs"),
    parts: &[
        &["ConnectedScatterChart", "root", "figure with a screen-reader table of every moment"],
        &["", "line", "The path in time order; draws along its length"],
        &["", "dot", "One moment; data-end is start or end for the first and last"],
        &["", "point-label", "Time labels on a few points, always the first and last"],
        &["", "axis-title", "Names of the two measures"],
        &["", "tooltip", "The moment and both values"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Earlier or later moment"],
        &["Home / End", "First or last moment"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["data", "Vec<Moment>", "Required. Moment::new(label, x, y), in time order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["x_label / y_label", "String", "Required. Names of the two measures"],
        &["height", "u16", "Plot height in px. Default 300"],
        &["time_labels", "usize", "Most points to label with their time. Default 6"],
        &["x_scale / y_scale", "ScaleKind", "Linear (default), Log or Symlog"],
        &["format_x / format_y", "Fn(f64) -> String", "Format each axis, the tooltip and table"],
        &["annotations", "Vec<Annotation>", "Reference lines, bands and points in data units"],
    ],
};
