use hydroxy_leptos::strip_plot_chart::{Annotation, Group, StripPlotChart};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let groups = vec![
        Group::new(
            "US East",
            [
                182.0, 195.0, 201.0, 188.0, 210.0, 224.0, 176.0, 199.0, 240.0, 205.0, 192.0, 186.0, 215.0, 230.0,
                198.0, 310.0, 207.0, 189.0, 221.0, 203.0,
            ],
        ),
        Group::new(
            "EU West",
            [
                142.0, 151.0, 160.0, 138.0, 149.0, 172.0, 155.0, 147.0, 166.0, 158.0, 144.0, 139.0, 181.0, 152.0,
                161.0, 148.0, 290.0, 157.0, 150.0, 163.0,
            ],
        ),
        Group::new(
            "Asia Pacific",
            [
                260.0, 284.0, 271.0, 299.0, 255.0, 318.0, 276.0, 290.0, 266.0, 305.0, 281.0, 340.0, 270.0, 262.0,
                295.0, 287.0, 412.0, 279.0, 301.0, 268.0,
            ],
        ),
    ];

    view! {
        <StripPlotChart
            groups=groups
            label="API response time by region"
            annotations=vec![Annotation::x_line(300.0, "SLO")]
            format=|v: f64| format!("{v:.0} ms")
        />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "strip-plot-chart",
    name: "Strip plot",
    group: "Points",
    summary: "Every value of each group as a dot on one row, spread by a small fixed jitter so repeated values stay visible. Good for modest samples where each observation matters.",
    demo,
    leptos: include_str!("strip_plot_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/strip_plot_chart.rs"),
    parts: &[
        &["StripPlotChart", "root", "figure with a screen-reader table of every value"],
        &["", "overlay", "Focusable plot; arrow keys step through dots"],
        &["", "dot", "One per value"],
        &["", "tooltip", "Group and value of the active dot"],
        &["", "annotation", "Reference line, band or point"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous dot, row by row"],
        &["Home / End", "First or last dot"],
        &["Escape", "Clear"],
    ],
    props: &[
        &["groups", "Vec<Group>", "Required. Group::new(label, values), one row each"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["jitter", "f64", "Share of each row the dots spread over. Default 0.6"],
        &["scale", "ScaleKind", "Linear, Log or Symlog value axis. Default Linear"],
        &["annotations", "Vec<Annotation>", "X in data values, y in row indices"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
