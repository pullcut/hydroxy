use dioxus::prelude::*;
use hydroxy_dioxus::tree_chart::{Orientation, Tree, TreeChart};

fn team(lead: &str, people: &[&str]) -> Tree {
    Tree::branch(lead, people.iter().map(|p| Tree::leaf(*p, 1.0)).collect())
}

#[component]
pub fn TreeChartDemo() -> Element {
    // Reporting lines; each person counts as one, so a branch's value is its headcount.
    let org = Tree::branch(
        "Priya (CEO)",
        vec![
            Tree::branch("Marcus (CTO)", vec![team("Lena", &["Tom", "Aisha", "Ravi"]), team("Diego", &["Mei", "Sam"])]),
            team("Hannah (CFO)", &["Oliver", "Grace"]),
            Tree::branch("Kofi (COO)", vec![team("Ines", &["Yuki", "Ben", "Zara"]), Tree::leaf("Noah", 1.0)]),
        ],
    );

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            TreeChart { root: org.clone(), label: "Reporting lines", format: |v: f64| format!("{v:.0} people") }
            TreeChart { root: org, label: "Reporting lines, left to right", orientation: Orientation::Horizontal, height: 420 }
        }
    }
}
