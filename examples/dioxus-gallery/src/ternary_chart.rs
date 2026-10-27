use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::ternary_chart::{TernaryChart, TernaryGroup, TernaryPoint};

#[component]
pub fn TernaryChartDemo() -> Element {
    let groups = vec![
        TernaryGroup::new(
            "Field A",
            [
                TernaryPoint::new("A1", 42.0, 38.0, 20.0),
                TernaryPoint::new("A2", 45.0, 35.0, 20.0),
                TernaryPoint::new("A3", 38.0, 40.0, 22.0),
                TernaryPoint::new("A4", 50.0, 32.0, 18.0),
            ],
        ),
        TernaryGroup::new(
            "Field B",
            [
                TernaryPoint::new("B1", 20.0, 25.0, 55.0),
                TernaryPoint::new("B2", 15.0, 30.0, 55.0),
                TernaryPoint::new("B3", 22.0, 20.0, 58.0),
                TernaryPoint::new("B4", 18.0, 34.0, 48.0),
            ],
        ),
        TernaryGroup::new(
            "Field C",
            [
                TernaryPoint::new("C1", 70.0, 20.0, 10.0),
                TernaryPoint::new("C2", 65.0, 27.0, 8.0),
                TernaryPoint::new("C3", 75.0, 15.0, 10.0),
            ],
        ),
    ];

    rsx! {
        ChartFrame { title: "Soil texture", description: "Sand, silt and clay by weight for each core sample.",
            TernaryChart {
                groups,
                label: "Soil samples by texture",
                a_label: "Sand",
                b_label: "Silt",
                c_label: "Clay",
                format: move |v: f64| format!("{v:.0} g"),
            }
        }
    }
}
