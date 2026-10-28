use dioxus::prelude::*;
use hydroxy_dioxus::chord_chart::ChordChart;

#[component]
pub fn ChordChartDemo() -> Element {
    let plans: Vec<String> =
        ["Free", "Starter", "Team", "Business", "Enterprise"].iter().map(|s| (*s).to_string()).collect();
    // Accounts moving between plans over the year: row = from, column = to.
    let moves = vec![
        vec![0.0, 420.0, 160.0, 30.0, 4.0],
        vec![180.0, 0.0, 310.0, 60.0, 8.0],
        vec![40.0, 95.0, 0.0, 140.0, 22.0],
        vec![6.0, 12.0, 48.0, 0.0, 35.0],
        vec![0.0, 2.0, 5.0, 14.0, 0.0],
    ];

    rsx! {
        ChordChart { names: plans, matrix: moves, label: "Accounts moving between plans" }
    }
}
