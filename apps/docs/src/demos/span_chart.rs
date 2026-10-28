use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::span_chart::*;

pub fn demo() -> AnyView {
    let roles: Vec<String> =
        ["Junior designer", "Product designer", "Senior designer", "Design lead", "Head of design"].map(String::from).to_vec();
    // Each band runs from the lowest to the highest offer; the marker is the median.
    let bands = vec![
        Span::new(32.0, 41.0).with_marker(36.5),
        Span::new(42.0, 58.0).with_marker(49.0),
        Span::new(55.0, 74.0).with_marker(63.0),
        Span::new(70.0, 92.0).with_marker(80.0),
        Span::new(88.0, 125.0).with_marker(104.0),
    ];
    let cities: Vec<String> = ["Reykjavík", "Edinburgh", "Lisbon", "Athens"].map(String::from).to_vec();
    let july = vec![Span::new(9.0, 14.0), Span::new(11.0, 19.0), Span::new(18.0, 28.0), Span::new(23.0, 33.0)];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Salary bands" description="Offers made in 2025, thousands per year. The dot is the median offer.">
                <SpanChart rows=roles data=bands label="Salary band by design role" marker_label="Median" format=|v: f64| format!("£{v:.0}k") />
            </ChartFrame>
            <ChartFrame title="July temperatures" description="Typical overnight low to daytime high.">
                <SpanChart
                    rows=cities
                    data=july
                    label="Typical July temperature range by city"
                    format=|v: f64| format!("{v:.0}°C")
                    annotations=vec![Annotation::y_line(25.0, "Beach weather")]
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "span-chart",
    name: "Span chart",
    group: "Bars",
    summary: "A bar per category from a start value to an end value, such as a salary band or a temperature range, with both ends labelled. An optional marker shows a point inside the range, such as the median.",
    demo,
    leptos: include_str!("span_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/span_chart.rs"),
    parts: &[
        &["SpanChart", "root", "figure with a screen-reader table of every range"],
        &["", "bar", "The range, extending from its start"],
        &["", "end-label", "The start and end values, either side of the bar"],
        &["", "marker", "The point of interest inside the range"],
        &["", "x-label", "Row names, down the left"],
        &["", "tooltip", "Start, end, spread and the marker for the row"],
        &["", "legend", "Range and marker, shown when there are markers"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Previous or next row"],
        &["Home / End", "First or last row"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["rows", "Vec<String>", "Required. One label per row"],
        &["data", "Vec<Span>", "Required. Span::new(start, end), optionally .with_marker(value)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["marker_label", "String", "What the marker means, such as Median. Default Marker"],
        &["height", "u16", "Plot height in px. Default 36 per row"],
        &["format", "Fn(f64) -> String", "Formats ticks, end labels, tooltip and table"],
        &["annotations", "Vec<Annotation>", "y_line and y_band mark values (drawn vertically); x is a row index"],
    ],
};
