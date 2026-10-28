use leptos::prelude::*;
use hydroxy_leptos::violin_chart::{Annotation, Group, ViolinChart};

pub fn demo() -> AnyView {
    let groups = vec![
        Group::new("Platform", [12.0, 14.0, 15.0, 15.5, 16.0, 17.0, 18.0, 19.0, 21.0, 22.0, 41.0, 16.5, 17.5, 14.5, 20.0]),
        Group::new("Payments", [18.0, 22.0, 25.0, 27.0, 28.0, 30.0, 33.0, 35.0, 38.0, 44.0, 26.0, 29.0, 31.0, 24.0, 36.0]),
        Group::new("Search", [8.0, 9.0, 9.5, 10.0, 11.0, 12.0, 12.5, 14.0, 15.0, 31.0, 10.5, 11.5, 9.0, 13.0, 10.0]),
        Group::new("Mobile", [20.0, 26.0, 29.0, 31.0, 34.0, 36.0, 40.0, 47.0, 55.0, 33.0, 38.0, 28.0, 42.0, 35.0, 30.0]),
    ];

    view! {
        <ViolinChart
            groups=groups
            label="Deploy time by team"
            annotations=vec![Annotation::y_line(30.0, "Target")]
            format=|v: f64| format!("{v:.0} min")
        />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "violin-chart",
    name: "Violin chart",
    group: "Distributions",
    summary: "Each group's density mirrored about a centre line, with its median marked and an optional quartile box inside. Shows where values bunch up, which a box plot alone hides.",
    demo,
    leptos: include_str!("violin_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/violin_chart.rs"),
    parts: &[
        &["ViolinChart", "root", "figure with a screen-reader table of every observation"],
        &["", "overlay", "Focusable plot; arrow keys move between groups"],
        &["", "area", "Mirrored density per group"],
        &["", "box", "First to third quartile (inner_box)"],
        &["", "whisker", "Range within 1.5 IQR (inner_box)"],
        &["", "median", "Dot at the median"],
        &["", "tooltip", "Count, median, quartiles and bandwidth"],
        &["", "annotation", "Reference line, band or point"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous group"],
        &["Home / End", "First or last group"],
        &["Escape", "Clear"],
    ],
    props: &[
        &["groups", "Vec<Group>", "Required. Group::new(label, values), left to right"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["inner_box", "bool", "Draw the quartile box and whiskers. Default true"],
        &["bandwidth", "f64", "Kernel bandwidth in data units. Default: Silverman's rule"],
        &["annotations", "Vec<Annotation>", "Y in data values, x in group indices"],
        &["height", "u16", "Plot height in px. Default 260"],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
    ],
};
