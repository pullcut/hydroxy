use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::polar_area_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let months = [
        ("Jan", 14.0),
        ("Feb", 11.0),
        ("Mar", 9.0),
        ("Apr", 6.0),
        ("May", 4.0),
        ("Jun", 3.0),
        ("Jul", 5.0),
        ("Aug", 7.0),
        ("Sep", 8.0),
        ("Oct", 12.0),
        ("Nov", 17.0),
        ("Dec", 21.0),
    ];
    let data: Vec<Datum> = months.iter().map(|(m, v)| Datum::new(*m, *v)).collect();

    view! {
        <ChartFrame title="Incidents" description="Sev-2 and above, by month. Winter peaks with holiday traffic.">
            <PolarAreaChart data=data label="Incidents by month" />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "polar-area-chart",
    name: "Polar area chart",
    group: "Radial",
    summary: "A Nightingale rose: every category gets the same angle and its radius encodes the value. Suits cycles such as months or hours of the day.",
    demo,
    leptos: include_str!("polar_area_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/polar_area_chart.rs"),
    parts: &[
        &["PolarAreaChart", "root", "figure with a screen-reader table of every category"],
        &["", "plot", "Focusable rose; arrow keys step through wedges"],
        &["", "ring", "Reference rings at nice values, labelled up the middle"],
        &["", "wedge", "One per category, growing out from the centre"],
        &["", "category", "Category name outside its wedge"],
        &["", "tooltip", "Category and value of the active wedge"],
    ],
    keys: &[
        &["ArrowRight / ArrowDown", "Next wedge, clockwise"],
        &["ArrowLeft / ArrowUp", "Previous wedge"],
        &["Home / End", "First or last wedge"],
        &["Escape", "Clear the active wedge"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Categories clockwise from 12 o'clock"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Rose diameter in px. Default 280"],
        &["format", "Fn(f64) -> String", "Formats ring labels, tooltip and table"],
    ],
};
