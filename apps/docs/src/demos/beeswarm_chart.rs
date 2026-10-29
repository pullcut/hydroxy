use hydroxy_leptos::beeswarm_chart::{BeeswarmChart, Group};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let groups = vec![
        Group::new(
            "Free",
            [
                3.0, 5.0, 4.0, 7.0, 2.0, 6.0, 8.0, 4.0, 5.0, 3.0, 9.0, 6.0, 4.0, 12.0, 5.0, 7.0, 3.0, 6.0, 4.0, 5.0,
                8.0, 2.0, 6.0, 15.0, 4.0,
            ],
        ),
        Group::new(
            "Pro",
            [
                12.0, 18.0, 15.0, 22.0, 9.0, 25.0, 17.0, 14.0, 20.0, 31.0, 16.0, 19.0, 13.0, 24.0, 21.0, 18.0, 27.0,
                15.0, 11.0, 23.0, 19.0, 16.0, 40.0, 20.0, 14.0,
            ],
        ),
        Group::new(
            "Team",
            [
                28.0, 35.0, 31.0, 42.0, 25.0, 38.0, 33.0, 47.0, 30.0, 36.0, 29.0, 52.0, 34.0, 39.0, 27.0, 44.0, 32.0,
                37.0, 41.0, 26.0, 60.0, 35.0, 33.0, 38.0, 30.0,
            ],
        ),
    ];

    view! { <BeeswarmChart groups=groups label="Session length by plan" format=|v: f64| format!("{v:.0} min") /> }
        .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "beeswarm-chart",
    name: "Beeswarm chart",
    group: "Points",
    summary: "Every value as a dot, packed outwards from the row's centre line where values crowd so none overlap. The swarm's outline shows the distribution while each observation stays readable.",
    demo,
    leptos: include_str!("beeswarm_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/beeswarm_chart.rs"),
    parts: &[
        &["BeeswarmChart", "root", "figure with a screen-reader table of every value"],
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
        &["groups", "Vec<Group>", "Required. One row each; pass one group for a single swarm"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["scale", "ScaleKind", "Linear, Log or Symlog value axis. Default Linear"],
        &["annotations", "Vec<Annotation>", "X in data values, y in row indices"],
        &["height", "u16", "Plot height in px. Default 260"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
