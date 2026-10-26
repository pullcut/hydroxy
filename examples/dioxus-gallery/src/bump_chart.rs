use dioxus::prelude::*;
use oxide_dioxus::bump_chart::{BumpChart, Series};

#[component]
pub fn BumpChartDemo() -> Element {
    let weeks: Vec<String> = ["Wk 1", "Wk 2", "Wk 3", "Wk 4", "Wk 5", "Wk 6"].map(String::from).to_vec();
    let teams = vec![
        Series::new("Falcons", [42.0, 55.0, 61.0, 48.0, 70.0, 66.0]),
        Series::new("Otters", [51.0, 47.0, 58.0, 63.0, 52.0, 71.0]),
        Series::new("Herons", [38.0, 60.0, 49.0, 66.0, 64.0, 58.0]),
        Series::new("Badgers", [60.0, 44.0, 52.0, 41.0, 45.0, 49.0]),
        Series::new("Kestrels", [35.0, 39.0, 66.0, 55.0, 68.0, 62.0]),
    ];

    rsx! {
        BumpChart { series: teams, labels: weeks, label: "Team ranking by weekly points" }
    }
}
