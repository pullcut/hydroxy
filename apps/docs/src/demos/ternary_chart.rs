use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::ternary_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let groups = vec![
        TernaryGroup::new(
            "Field A",
            [
                TernaryPoint::new("A1", 42.0, 38.0, 20.0),
                TernaryPoint::new("A2", 45.0, 35.0, 20.0),
                TernaryPoint::new("A3", 38.0, 40.0, 22.0),
                TernaryPoint::new("A4", 50.0, 32.0, 18.0),
            ],
        ),
        TernaryGroup::new(
            "Field B",
            [
                TernaryPoint::new("B1", 20.0, 25.0, 55.0),
                TernaryPoint::new("B2", 15.0, 30.0, 55.0),
                TernaryPoint::new("B3", 22.0, 20.0, 58.0),
                TernaryPoint::new("B4", 18.0, 34.0, 48.0),
            ],
        ),
        TernaryGroup::new(
            "Field C",
            [
                TernaryPoint::new("C1", 70.0, 20.0, 10.0),
                TernaryPoint::new("C2", 65.0, 27.0, 8.0),
                TernaryPoint::new("C3", 75.0, 15.0, 10.0),
            ],
        ),
    ];

    view! {
        <ChartFrame title="Soil texture" description="Sand, silt and clay by weight for each core sample.">
            <TernaryChart groups=groups label="Soil samples by texture" a_label="Sand" b_label="Silt" c_label="Clay" format=|v: f64| format!("{v:.0} g") />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "ternary-chart",
    name: "Ternary chart",
    group: "Points",
    summary: "Compositions of three parts as points in a triangle. Each corner is all of one part; gridlines every 20% read off each share. Values are normalised, so any units work.",
    demo,
    leptos: include_str!("ternary_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/ternary_chart.rs"),
    parts: &[
        &["TernaryChart", "root", "figure with a screen-reader table of every point"],
        &["", "plot", "Focusable triangle; arrow keys step through points"],
        &["", "gridline", "Lines of equal share for each part, every 20%"],
        &["", "tick", "Share labels: A on the left edge, B along the bottom, C on the right"],
        &["", "corner", "Part names at their corners"],
        &["", "dot", "One per point, in its group colour"],
        &["", "tooltip", "Each share with its raw value"],
        &["", "legend", "Group names, shown for two or more groups"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous point"],
        &["Home / End", "First or last point"],
        &["Escape", "Clear the active point"],
    ],
    props: &[
        &[
            "groups",
            "Vec<TernaryGroup>",
            "Required. TernaryGroup::new(name, points); TernaryPoint::new(label, a, b, c)",
        ],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["a_label / b_label / c_label", "String", "Required. Names of the three parts"],
        &["size", "u16", "Triangle width in px. Default 360"],
        &["format", "Fn(f64) -> String", "Formats raw values in the tooltip and table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
