use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::voronoi_treemap_chart::{Tree, VoronoiTreemapChart};

fn group(name: &str, parts: &[(&str, f64)]) -> Tree {
    Tree::branch(name, parts.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

#[component]
pub fn VoronoiTreemapChartDemo() -> Element {
    // Last month's cloud bill in thousands of dollars.
    let bill = Tree::branch(
        "Cloud spend",
        vec![
            group(
                "Compute",
                &[("Virtual machines", 48.2), ("Kubernetes", 36.5), ("Serverless", 9.8), ("GPU jobs", 21.4)],
            ),
            group("Storage", &[("Object storage", 18.6), ("Block volumes", 11.2), ("Backups", 6.9)]),
            group("Data", &[("Warehouse", 27.3), ("Postgres", 14.1), ("Cache", 5.2)]),
            group("Network", &[("Egress", 12.8), ("CDN", 7.4), ("Load balancers", 4.6)]),
            group("Observability", &[("Logs", 8.9), ("Metrics", 4.2), ("Traces", 2.7)]),
        ],
    );

    rsx! {
        ChartFrame { title: "Cloud spend by service", description: "Last month, $k. Each cell's area is its share of the bill.",
            VoronoiTreemapChart { root: bill, label: "Cloud spend by service, last month", format: |v: f64| format!("${v:.1}k") }
        }
    }
}
