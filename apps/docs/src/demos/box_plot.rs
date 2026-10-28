use leptos::prelude::*;
use hydroxy_leptos::box_plot::{BoxPlot, Group};

pub fn demo() -> AnyView {
    let groups = vec![
        Group::new("Platform", [12.0, 14.0, 15.0, 15.5, 16.0, 17.0, 18.0, 19.0, 21.0, 22.0, 41.0]),
        Group::new("Payments", [18.0, 22.0, 25.0, 27.0, 28.0, 30.0, 33.0, 35.0, 38.0, 44.0]),
        Group::new("Search", [8.0, 9.0, 9.5, 10.0, 11.0, 12.0, 12.5, 14.0, 15.0, 2.0, 31.0]),
        Group::new("Mobile", [20.0, 26.0, 29.0, 31.0, 34.0, 36.0, 40.0, 47.0, 55.0]),
    ];

    view! { <BoxPlot groups=groups label="Deploy time by team" format=|v: f64| format!("{v:.0} min") /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "box-plot",
    name: "Box plot",
    group: "Distributions",
    summary: "Compare the spread of several groups. Each box shows the quartiles and median, whiskers reach to the furthest value within 1.5 times the interquartile range, and values beyond them are drawn as outliers. Hover, touch or arrow-key to a box to read its numbers.",
    demo,
    leptos: include_str!("box_plot.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/box_plot.rs"),
    parts: &[
        &["BoxPlot", "root", "figure with a screen-reader table of each group's numbers"],
        &["", "columns", "Focusable group of boxes; arrow keys step between them"],
        &["", "column", "One per group; `data-active` and its tooltip shown when picked"],
        &["", "box", "First to third quartile"],
        &["", "median", "Line across the box"],
        &["", "whisker", "Range within 1.5 IQR, with a cap at each end"],
        &["", "outlier", "Hollow dot for each value beyond the whiskers"],
        &["", "tooltip", "Whiskers, quartiles, median and outlier count of the picked group"],
        &["", "sr-live", "Announces the picked group's numbers"],
        &["", "sr-table", "Every group's count, quartiles, whiskers and outliers"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Step to the next or previous group"],
        &["Home / End", "Jump to the first or last group"],
        &["Escape", "Clear the picked group"],
    ],
    props: &[
        &["groups", "Vec<Group>", "Required. Group::new(label, values); non-finite values are ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 240"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
    ],
};
