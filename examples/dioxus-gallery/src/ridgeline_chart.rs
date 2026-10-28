use dioxus::prelude::*;
use hydroxy_dioxus::ridgeline_chart::{Group, RidgelineChart};

#[component]
pub fn RidgelineChartDemo() -> Element {
    let groups = vec![
        Group::new("January", [6.0, 8.0, 7.0, 5.0, 9.0, 4.0, 7.0, 8.0, 6.0, 10.0, 7.0, 5.0, 3.0, 8.0, 7.0, 6.0]),
        Group::new("February", [7.0, 9.0, 8.0, 6.0, 10.0, 5.0, 8.0, 11.0, 7.0, 9.0, 6.0, 8.0, 4.0, 9.0, 10.0, 7.0]),
        Group::new("March", [10.0, 12.0, 9.0, 11.0, 14.0, 8.0, 13.0, 10.0, 12.0, 11.0, 15.0, 9.0, 10.0, 13.0, 11.0, 12.0]),
        Group::new("April", [13.0, 15.0, 12.0, 16.0, 14.0, 11.0, 17.0, 13.0, 15.0, 14.0, 18.0, 12.0, 16.0, 13.0, 15.0, 14.0]),
        Group::new("May", [16.0, 18.0, 17.0, 20.0, 15.0, 19.0, 22.0, 17.0, 18.0, 16.0, 21.0, 18.0, 19.0, 17.0, 23.0, 18.0]),
        Group::new("June", [19.0, 21.0, 20.0, 24.0, 18.0, 22.0, 26.0, 20.0, 23.0, 21.0, 25.0, 19.0, 22.0, 20.0, 27.0, 21.0]),
    ];

    rsx! {
        RidgelineChart { groups, label: "Daily high temperature by month", format: move |v: f64| format!("{v:.0}°C") }
    }
}
