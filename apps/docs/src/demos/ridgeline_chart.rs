use hydroxy_leptos::ridgeline_chart::{Group, RidgelineChart};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let groups = vec![
        Group::new("January", [6.0, 8.0, 7.0, 5.0, 9.0, 4.0, 7.0, 8.0, 6.0, 10.0, 7.0, 5.0, 3.0, 8.0, 7.0, 6.0]),
        Group::new("February", [7.0, 9.0, 8.0, 6.0, 10.0, 5.0, 8.0, 11.0, 7.0, 9.0, 6.0, 8.0, 4.0, 9.0, 10.0, 7.0]),
        Group::new(
            "March",
            [10.0, 12.0, 9.0, 11.0, 14.0, 8.0, 13.0, 10.0, 12.0, 11.0, 15.0, 9.0, 10.0, 13.0, 11.0, 12.0],
        ),
        Group::new(
            "April",
            [13.0, 15.0, 12.0, 16.0, 14.0, 11.0, 17.0, 13.0, 15.0, 14.0, 18.0, 12.0, 16.0, 13.0, 15.0, 14.0],
        ),
        Group::new(
            "May",
            [16.0, 18.0, 17.0, 20.0, 15.0, 19.0, 22.0, 17.0, 18.0, 16.0, 21.0, 18.0, 19.0, 17.0, 23.0, 18.0],
        ),
        Group::new(
            "June",
            [19.0, 21.0, 20.0, 24.0, 18.0, 22.0, 26.0, 20.0, 23.0, 21.0, 25.0, 19.0, 22.0, 20.0, 27.0, 21.0],
        ),
    ];

    view! { <RidgelineChart groups=groups label="Daily high temperature by month" format=|v: f64| format!("{v:.0}°C") /> }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "ridgeline-chart",
    name: "Ridgeline chart",
    group: "Distributions",
    summary: "One density curve per group, stacked in overlapping rows. Each ridge is scaled to its own peak, so it reads shape and shift across many groups, such as months or cohorts, rather than absolute counts.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("ridgeline_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/ridgeline_chart.rs"),
    parts: &[
        &["RidgelineChart", "root", "figure with a screen-reader table of every observation"],
        &["", "overlay", "Focusable plot; arrow keys move between rows"],
        &["", "area", "Filled ridge per group"],
        &["", "line", "Ridge outline"],
        &["", "tooltip", "Count, median, peak and bandwidth of the active row"],
        &["", "annotation", "Reference line, band or point"],
    ],
    keys: &[
        &["ArrowDown / ArrowUp", "Next or previous row (Right and Left work too)"],
        &["Home / End", "First or last row"],
        &["Escape", "Clear"],
    ],
    props: &[
        &["groups", "Vec<Group>", "Required. One ridge each, from the top"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["overlap", "f64", "How far a ridge rises into the rows above, in rows. Default 1"],
        &["bandwidth", "f64", "Kernel bandwidth in data units. Default: Silverman's rule"],
        &["annotations", "Vec<Annotation>", "X in data values, y in row indices"],
        &["height", "u16", "Plot height in px. Default 280"],
        &["format", "Fn(f64) -> String", "Formats value ticks, tooltip and table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
