use dioxus::prelude::*;
use hydroxy_dioxus::violin_chart::{Annotation, Group, ViolinChart};

#[component]
pub fn ViolinChartDemo() -> Element {
    let groups = vec![
        Group::new(
            "Platform",
            [12.0, 14.0, 15.0, 15.5, 16.0, 17.0, 18.0, 19.0, 21.0, 22.0, 41.0, 16.5, 17.5, 14.5, 20.0],
        ),
        Group::new(
            "Payments",
            [18.0, 22.0, 25.0, 27.0, 28.0, 30.0, 33.0, 35.0, 38.0, 44.0, 26.0, 29.0, 31.0, 24.0, 36.0],
        ),
        Group::new("Search", [8.0, 9.0, 9.5, 10.0, 11.0, 12.0, 12.5, 14.0, 15.0, 31.0, 10.5, 11.5, 9.0, 13.0, 10.0]),
        Group::new(
            "Mobile",
            [20.0, 26.0, 29.0, 31.0, 34.0, 36.0, 40.0, 47.0, 55.0, 33.0, 38.0, 28.0, 42.0, 35.0, 30.0],
        ),
    ];

    rsx! {
        ViolinChart {
            groups,
            label: "Deploy time by team",
            annotations: vec![Annotation::y_line(30.0, "Target")],
            format: move |v: f64| format!("{v:.0} min"),
        }
    }
}
