use hydroxy_leptos::alluvial_chart::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"].iter().map(|q| format!("{q} 2025")).collect();
    // Accounts that signed up in Q1, by plan at the end of each quarter.
    let accounts = vec![
        Cohort::new(["Free", "Free", "Free", "Free"], 820.0),
        Cohort::new(["Free", "Free", "Starter", "Starter"], 240.0),
        Cohort::new(["Free", "Starter", "Starter", "Pro"], 130.0),
        Cohort::new(["Free", "Starter", "Pro", "Pro"], 90.0),
        Cohort::new(["Free", "Churned", "Churned", "Churned"], 310.0),
        Cohort::new(["Free", "Free", "Churned", "Churned"], 180.0),
        Cohort::new(["Starter", "Starter", "Starter", "Starter"], 260.0),
        Cohort::new(["Starter", "Pro", "Pro", "Pro"], 150.0),
        Cohort::new(["Starter", "Starter", "Churned", "Churned"], 70.0),
        Cohort::new(["Starter", "Free", "Free", "Churned"], 40.0),
        Cohort::new(["Pro", "Pro", "Pro", "Pro"], 210.0),
        Cohort::new(["Pro", "Starter", "Starter", "Starter"], 35.0),
        Cohort::new(["Pro", "Pro", "Churned", "Churned"], 25.0),
    ];

    view! {
        <ChartFrame title="Plan moves" description="Accounts that signed up in Q1, by plan at each quarter's end. Most churn leaves from Free in the first quarter.">
            <AlluvialChart data=accounts steps=quarters label="Q1 2025 sign-ups by plan, quarter by quarter" />
        </ChartFrame>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "alluvial-chart",
    name: "Alluvial chart",
    group: "Relationships",
    summary: "How members move between categories across ordered steps. Each step stacks its categories in the same order, and ribbons carry the members from one step's category to the next's, coloured by where they came from. Hover a category to light its ribbons.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("alluvial_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/alluvial_chart.rs"),
    parts: &[
        &["AlluvialChart", "root", "figure; `data-active` while a category or ribbon is picked"],
        &["", "step", "A step's name above its column"],
        &["", "flow", "The ribbons and nodes, revealed left to right"],
        &["", "node", "A category at one step; `data-highlighted` when lit, `data-selected` when picked"],
        &["", "ribbon", "Members moving between two categories; `data-highlighted` when lit"],
        &["", "node-label", "Category name and count, beside nodes tall enough for them"],
        &["", "tooltip", "Members and shares for the picked category or ribbon"],
        &["", "sr-table", "Every ribbon with its steps, categories and count, for screen readers and CSV export"],
    ],
    keys: &[
        &["Arrow keys", "Step through the categories, then the ribbons"],
        &["Home / End", "First or last mark"],
        &["Escape", "Clear the picked mark"],
    ],
    props: &[
        &["data", "Vec<Cohort>", "Required. Members sharing a path, one category per step"],
        &["steps", "Vec<String>", "Required. Step names in order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["width", "u16", "Drawing width in px. Default 720"],
        &["height", "u16", "Drawing height in px. Default 360"],
        &["format", "Fn(f64) -> String", "Formats counts in labels, the tooltip and the table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
