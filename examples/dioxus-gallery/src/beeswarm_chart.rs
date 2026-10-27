use dioxus::prelude::*;
use oxide_dioxus::beeswarm_chart::{BeeswarmChart, Group};

#[component]
pub fn BeeswarmChartDemo() -> Element {
    let groups = vec![
        Group::new("Free", [3.0, 5.0, 4.0, 7.0, 2.0, 6.0, 8.0, 4.0, 5.0, 3.0, 9.0, 6.0, 4.0, 12.0, 5.0, 7.0, 3.0, 6.0, 4.0, 5.0, 8.0, 2.0, 6.0, 15.0, 4.0]),
        Group::new("Pro", [12.0, 18.0, 15.0, 22.0, 9.0, 25.0, 17.0, 14.0, 20.0, 31.0, 16.0, 19.0, 13.0, 24.0, 21.0, 18.0, 27.0, 15.0, 11.0, 23.0, 19.0, 16.0, 40.0, 20.0, 14.0]),
        Group::new("Team", [28.0, 35.0, 31.0, 42.0, 25.0, 38.0, 33.0, 47.0, 30.0, 36.0, 29.0, 52.0, 34.0, 39.0, 27.0, 44.0, 32.0, 37.0, 41.0, 26.0, 60.0, 35.0, 33.0, 38.0, 30.0]),
    ];

    rsx! {
        BeeswarmChart { groups, label: "Session length by plan", format: move |v: f64| format!("{v:.0} min") }
    }
}
