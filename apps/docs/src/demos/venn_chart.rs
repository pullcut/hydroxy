use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::venn_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    // Customers by the exact mix of places they bought from this year.
    let channels = vec![
        Intersection::new(["Web store"], 4120.0),
        Intersection::new(["Mobile app"], 2630.0),
        Intersection::new(["Retail stores"], 3380.0),
        Intersection::new(["Web store", "Mobile app"], 1840.0),
        Intersection::new(["Web store", "Retail stores"], 960.0),
        Intersection::new(["Mobile app", "Retail stores"], 410.0),
        Intersection::new(["Web store", "Mobile app", "Retail stores"], 520.0),
    ];

    view! {
        <ChartFrame title="Where customers buy" description="Circle areas match each channel's customers, and each overlap matches the customers two channels share.">
            <VennChart data=channels label="Customers by purchase channel" />
        </ChartFrame>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "venn-chart",
    name: "Venn chart",
    group: "Relationships",
    summary: "Two or three sets as circles whose areas match their sizes, with each pairwise overlap solved to the members they share. Every region carries its count; hover one to read its share.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("venn_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/venn_chart.rs"),
    parts: &[
        &["VennChart", "root", "figure; `data-active` while a region is picked"],
        &[
            "",
            "circle",
            "One set, a translucent fill with a surface ring; `data-highlighted` when the picked region is inside it",
        ],
        &["", "region", "The area inside exactly some sets, for hovering; `data-active` when picked"],
        &["", "region-value", "The region's member count; `data-empty` when it is zero"],
        &["", "set-name", "Set name and size, outside its circle"],
        &["", "tooltip", "Members and share of everyone; a set's size for a single-set region"],
        &["", "sr-table", "Every region with its count, for screen readers and CSV export"],
    ],
    keys: &[
        &["Arrow keys", "Step through the regions"],
        &["Home / End", "First or last region"],
        &["Escape", "Clear the picked region"],
    ],
    props: &[
        &[
            "data",
            "Vec<Intersection>",
            "Required. Members in exactly each combination of sets; the three largest sets are drawn",
        ],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["width", "u16", "Drawing width in px. Default 520"],
        &["height", "u16", "Drawing height in px. Default 360"],
        &["format", "Fn(f64) -> String", "Formats counts in labels, the tooltip and the table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
