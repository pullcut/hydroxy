use hydroxy_leptos::bump_chart::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let weeks: Vec<String> = ["Wk 1", "Wk 2", "Wk 3", "Wk 4", "Wk 5", "Wk 6"].map(String::from).to_vec();
    // Points scored each week; the chart ranks the teams at every step.
    let teams = vec![
        Series::new("Falcons", [42.0, 55.0, 61.0, 48.0, 70.0, 66.0]),
        Series::new("Otters", [51.0, 47.0, 58.0, 63.0, 52.0, 71.0]),
        Series::new("Herons", [38.0, 60.0, 49.0, 66.0, 64.0, 58.0]),
        Series::new("Badgers", [60.0, 44.0, 52.0, 41.0, 45.0, 49.0]),
        Series::new("Kestrels", [35.0, 39.0, 66.0, 55.0, 68.0, 62.0]),
    ];

    view! {
        // Hover a marker, or focus the chart and use the arrow keys, to read a rank.
        <BumpChart series=teams labels=weeks label="Team ranking by weekly points" />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bump-chart",
    name: "Bump chart",
    group: "Lines and areas",
    summary: "Show how items trade places over time. Values only decide the order at each step, so the chart answers who is ahead, not by how much. Hover, touch or focus a marker to read its rank and value; hover a legend item to pick its series out.",
    demo,
    leptos: include_str!("bump_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bump_chart.rs"),
    parts: &[
        &["BumpChart", "root", "figure holding the plot, legend, live region and table"],
        &["", "plot", "Focusable group around the SVG and tooltip"],
        &["", "marks", "The SVG; it scales to the container width"],
        &["", "series", "Group per item; `data-dim` while another series is picked"],
        &["", "line", "S-curve between ranks, in the series colour"],
        &["", "point", "Marker at each step; `data-active` when picked"],
        &["", "hit", "Transparent target around each marker for hover and touch"],
        &["", "name", "Direct label at both ends"],
        &["", "tooltip", "Rank and value at the picked step"],
        &["", "legend", "Names and colours, shown for two or more series"],
        &["", "sr-live", "Announces the picked rank"],
        &["", "sr-table", "Every rank and value, for screen readers and CSV export"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Move along the line"],
        &["ArrowUp / ArrowDown", "Move to the previous or next series"],
        &["Home / End", "Jump to the first or last step"],
        &["Escape", "Clear the picked marker"],
    ],
    props: &[
        &["series", "Vec<Series>", "Required. Series::new(name, values); values decide the rank"],
        &["labels", "Vec<String>", "Required. One label per column"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["higher_is_better", "bool", "Rank the highest value first. Default true"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip and the table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
