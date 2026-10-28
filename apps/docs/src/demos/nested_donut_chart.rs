use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::nested_donut_chart::*;

pub fn demo() -> AnyView {
    // Website sessions by source, in thousands, one ring per year.
    let year = |name: &str, values: [f64; 5]| {
        let sources = ["Search", "Direct", "Social", "Referral", "Email"];
        (name.to_string(), sources.iter().zip(values).map(|(s, v)| Datum::new(*s, v)).collect::<Vec<_>>())
    };
    let rings = series(&[year("2023", [412.0, 186.0, 94.0, 61.0, 38.0]), year("2024", [455.0, 204.0, 141.0, 58.0, 47.0]), year("2025", [468.0, 231.0, 212.0, 52.0, 66.0])]);

    view! {
        <ChartFrame title="Sessions by source" description="Thousands of sessions a year, 2023 in the middle and 2025 outside. Social has more than doubled its share.">
            <NestedDonutChart rings=rings label="Website sessions by source, 2023 to 2025" format=|v: f64| format!("{v:.0}k") />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "nested-donut-chart",
    name: "Nested donut chart",
    group: "Parts of a whole",
    summary: "Concentric rings, innermost first: one per series over the same categories, built with `series`, or one per level of a tree, built with `levels`, where children sit under their parent's sweep. The whole chart sweeps in clockwise as one motion.",
    demo,
    leptos: include_str!("nested_donut_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/nested_donut_chart.rs"),
    parts: &[
        &["NestedDonutChart", "root", "figure; `data-active` while a segment is picked"],
        &["", "plot", "Focusable square holding the rings, centre and tooltip"],
        &["", "ring", "One ring's segments; `data-ring` counts from 0 at the centre"],
        &["", "segment", "One part, a stroked arc; `data-active` when picked"],
        &["", "center", "The picked part's value and name, or the outer ring's total"],
        &["", "legend", "Categories in colour order"],
        &["", "ring-key", "Ring names from the centre out, when there are two or more"],
        &["", "sr-table", "Every segment with its ring, value and share, for screen readers and CSV export"],
    ],
    keys: &[
        &["Arrow keys", "Step through the segments, ring by ring from the centre"],
        &["Home / End", "First or last segment"],
        &["Escape", "Clear the picked segment"],
    ],
    props: &[
        &["rings", "Vec<Ring>", "Required. Innermost first; build with series(&[(name, data)]) or levels(&tree, depth)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["size", "u16", "Diameter in px. Default 280"],
        &["hole", "f64", "The empty centre's share of the radius. Default 0.42"],
        &["center_label", "String", "Caption under the centre figure. Default: the outer ring's name"],
        &["format", "Fn(f64) -> String", "Formats values in the centre, tooltip and table"],
    ],
};
