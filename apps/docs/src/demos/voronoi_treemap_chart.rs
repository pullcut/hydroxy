use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::voronoi_treemap_chart::*;

fn group(name: &str, parts: &[(&str, f64)]) -> Tree {
    Tree::branch(name, parts.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

pub fn demo() -> AnyView {
    // Last month's cloud bill in thousands of dollars.
    let bill = Tree::branch(
        "Cloud spend",
        vec![
            group("Compute", &[("Virtual machines", 48.2), ("Kubernetes", 36.5), ("Serverless", 9.8), ("GPU jobs", 21.4)]),
            group("Storage", &[("Object storage", 18.6), ("Block volumes", 11.2), ("Backups", 6.9)]),
            group("Data", &[("Warehouse", 27.3), ("Postgres", 14.1), ("Cache", 5.2)]),
            group("Network", &[("Egress", 12.8), ("CDN", 7.4), ("Load balancers", 4.6)]),
            group("Observability", &[("Logs", 8.9), ("Metrics", 4.2), ("Traces", 2.7)]),
        ],
    );

    view! {
        <ChartFrame title="Cloud spend by service" description="Last month, $k. Each cell's area is its share of the bill.">
            <VoronoiTreemapChart root=bill label="Cloud spend by service, last month" format=|v: f64| format!("${v:.1}k") />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "voronoi-treemap-chart",
    name: "Voronoi treemap",
    group: "Parts of a whole",
    summary: "A tree as nested weighted Voronoi cells inside a circle or a rectangle, each cell's area its share of its parent. Cells are coloured by top-level group and the larger ones are labelled. The layout is seeded, so the same data always draws the same picture.",
    demo,
    leptos: include_str!("voronoi_treemap_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/voronoi_treemap_chart.rs"),
    parts: &[
        &["VoronoiTreemapChart", "root", "figure; `data-active` while a cell is picked"],
        &["", "marks", "The focusable svg; `data-shape` is circle or rect"],
        &["", "cell", "One node's polygon; `data-depth` is its level, `data-leaf` when nothing is drawn inside it, `data-active` when picked"],
        &["", "border", "A branch's outline, heavier for higher levels"],
        &["", "label", "Name and value, in cells with room for them"],
        &["", "tooltip", "Value, share of the whole and top-level group"],
        &["", "legend", "The top-level groups in colour order, when there are two or more"],
        &["", "sr-table", "Every leaf with its path and value, for screen readers and CSV export"],
    ],
    keys: &[
        &["Arrow keys", "Step through the cells"],
        &["Home / End", "First or last cell"],
        &["Escape", "Clear the picked cell"],
    ],
    props: &[
        &["root", "Tree", "Required. Branch values come from their children"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["width", "u16", "Drawing width in px. Default 520"],
        &["height", "u16", "Drawing height in px. Default 520"],
        &["shape", "Shape", "Circle or Rect outline. Default Circle"],
        &["depth", "u8", "Levels drawn, 1 to 4. Default 2"],
        &["format", "Fn(f64) -> String", "Formats values in labels, the tooltip and the table"],
    ],
};
