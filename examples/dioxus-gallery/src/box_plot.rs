use dioxus::prelude::*;
use oxide_dioxus::box_plot::{BoxPlot, Group};

#[component]
pub fn BoxPlotDemo() -> Element {
    let groups = vec![
        Group::new("Platform", [12.0, 14.0, 15.0, 15.5, 16.0, 17.0, 18.0, 19.0, 21.0, 22.0, 41.0]),
        Group::new("Payments", [18.0, 22.0, 25.0, 27.0, 28.0, 30.0, 33.0, 35.0, 38.0, 44.0]),
        Group::new("Search", [8.0, 9.0, 9.5, 10.0, 11.0, 12.0, 12.5, 14.0, 15.0, 2.0, 31.0]),
        Group::new("Mobile", [20.0, 26.0, 29.0, 31.0, 34.0, 36.0, 40.0, 47.0, 55.0]),
    ];

    rsx! {
        BoxPlot { groups, label: "Deploy time by team", format: move |v: f64| format!("{v:.0} min") }
    }
}
