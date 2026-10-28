use dioxus::prelude::*;
use hydroxy_dioxus::bubble_pack_chart::{BubblePackChart, Tree};

fn folder(name: &str, files: &[(&str, f64)]) -> Tree {
    Tree::branch(name, files.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

#[component]
pub fn BubblePackChartDemo() -> Element {
    // Disk usage of a monorepo checkout, in MB.
    let repo = Tree::branch(
        "repo",
        vec![
            Tree::branch(
                "apps",
                vec![
                    folder("web", &[("node_modules", 412.0), ("dist", 38.0), ("src", 9.0)]),
                    folder("docs", &[("node_modules", 188.0), ("public", 64.0), ("content", 12.0)]),
                    folder("mobile", &[("ios", 96.0), ("android", 121.0), ("src", 14.0)]),
                ],
            ),
            folder("packages", &[("ui", 22.0), ("charts", 17.0), ("auth", 6.0), ("billing", 8.0)]),
            folder("assets", &[("video", 240.0), ("images", 86.0), ("fonts", 11.0)]),
            folder(".git", &[("objects", 302.0), ("lfs", 140.0)]),
        ],
    );

    rsx! {
        BubblePackChart {
            root: repo,
            label: "Disk usage by folder",
            format: |v: f64| format!("{v:.0} MB"),
        }
    }
}
