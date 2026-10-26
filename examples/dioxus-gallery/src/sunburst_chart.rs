use dioxus::prelude::*;
use oxide_dioxus::sunburst_chart::{SunburstChart, Tree};

fn team(name: &str, parts: &[(&str, f64)]) -> Tree {
    Tree::branch(name, parts.iter().map(|(n, v)| Tree::leaf(*n, *v)).collect())
}

#[component]
pub fn SunburstChartDemo() -> Element {
    let spend = Tree::branch(
        "Spend",
        vec![
            Tree::branch(
                "Engineering",
                vec![
                    team("Platform", &[("Infra", 260.0), ("Tooling", 160.0)]),
                    team("Product", &[("Web", 220.0), ("Mobile", 160.0)]),
                    Tree::leaf("Data", 210.0),
                    Tree::leaf("Security", 90.0),
                ],
            ),
            team("Sales", &[("Enterprise", 300.0), ("SMB", 180.0), ("Partners", 90.0)]),
            team("Marketing", &[("Brand", 150.0), ("Growth", 210.0), ("Events", 80.0)]),
            team("Support", &[("Tier 1", 140.0), ("Tier 2", 90.0)]),
            team("Operations", &[("People", 110.0), ("Finance", 70.0), ("Legal", 40.0)]),
        ],
    );

    rsx! {
        SunburstChart { root: spend, label: "Annual spend by team" }
    }
}
