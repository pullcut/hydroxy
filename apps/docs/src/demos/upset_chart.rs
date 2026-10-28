use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::upset_chart::*;

pub fn demo() -> AnyView {
    // Customers by the exact set of notification channels they keep switched on.
    let channels = vec![
        Intersection::new(["Email"], 4120.0),
        Intersection::new(["Email", "Push"], 2860.0),
        Intersection::new(["Push"], 1940.0),
        Intersection::new(["Email", "In-app"], 1310.0),
        Intersection::new(["Email", "Push", "In-app"], 1180.0),
        Intersection::new(["In-app"], 760.0),
        Intersection::new(["Email", "SMS"], 640.0),
        Intersection::new(["Push", "In-app"], 590.0),
        Intersection::new(["Email", "Slack"], 470.0),
        Intersection::new(["SMS"], 380.0),
        Intersection::new(["Email", "Push", "SMS"], 310.0),
        Intersection::new(["Slack"], 260.0),
        Intersection::new(["Email", "Slack", "In-app"], 220.0),
        Intersection::new(["Push", "SMS"], 140.0),
        Intersection::new(["Email", "Push", "In-app", "Slack"], 90.0),
    ];

    view! {
        <ChartFrame title="Notification channels" description="Customers by the exact combination of channels they keep switched on. Email alone is still the largest group.">
            <UpsetChart data=channels label="Customers by notification channels switched on" />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "upset-chart",
    name: "UpSet chart",
    group: "Relationships",
    summary: "How many sets overlap, past the three a Venn diagram can hold. Bars on top count each exact combination, the dot matrix shows which sets it joins, and bars on the left give each set's size. Hover a column to light its sets, or a set to light every combination that includes it.",
    demo,
    leptos: include_str!("upset_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/upset_chart.rs"),
    parts: &[
        &["UpsetChart", "root", "figure; `data-active` while a column or set is picked"],
        &["", "column", "One intersection: its bar, count and dots; `data-highlighted` or `data-dim` while something is picked"],
        &["", "bar", "Members in exactly these sets, grown from the baseline"],
        &["", "dot", "One set in the matrix; `data-member` when the column includes it"],
        &["", "connector", "Joins a column's member dots"],
        &["", "set", "One set's row: its size bar, value and name"],
        &["", "note", "Says how many small intersections the column limit left out"],
        &["", "tooltip", "Members, share of everyone and set count, or a set's size"],
        &["", "sr-table", "Every intersection with its count, for screen readers and CSV export"],
    ],
    keys: &[
        &["Arrow keys", "Step through the columns, then the sets"],
        &["Home / End", "First or last mark"],
        &["Escape", "Clear the picked mark"],
    ],
    props: &[
        &["data", "Vec<Intersection>", "Required. Members in exactly each combination of sets"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["width", "u16", "Drawing width in px; the height follows the set count. Default 720"],
        &["max_columns", "usize", "Most intersections to draw, largest first. Default 16"],
        &["format", "Fn(f64) -> String", "Formats counts in labels, the tooltip and the table"],
    ],
};
