use dioxus::prelude::*;
use hydroxy_dioxus::treemap_chart::{Tree, TreemapChart};

fn team(name: &str, parts: &[(&str, f64)]) -> Tree {
    Tree::branch(name, parts.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

#[component]
pub fn TreemapChartDemo() -> Element {
    let spend = Tree::branch(
        "Spend",
        vec![
            team("Engineering", &[("Platform", 420.0), ("Product", 380.0), ("Data", 210.0), ("Security", 90.0)]),
            team("Sales", &[("Enterprise", 300.0), ("SMB", 180.0), ("Partners", 90.0)]),
            team("Marketing", &[("Brand", 150.0), ("Growth", 210.0), ("Events", 80.0)]),
            team("Support", &[("Tier 1", 140.0), ("Tier 2", 90.0)]),
            team("Operations", &[("People", 110.0), ("Finance", 70.0), ("Legal", 40.0)]),
        ],
    );

    rsx! {
        TreemapChart { root: spend, label: "Annual spend by team" }
    }
}
