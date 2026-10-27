use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::polar_area_chart::{Datum, PolarAreaChart};

#[component]
pub fn PolarAreaChartDemo() -> Element {
    let months = [
        ("Jan", 14.0),
        ("Feb", 11.0),
        ("Mar", 9.0),
        ("Apr", 6.0),
        ("May", 4.0),
        ("Jun", 3.0),
        ("Jul", 5.0),
        ("Aug", 7.0),
        ("Sep", 8.0),
        ("Oct", 12.0),
        ("Nov", 17.0),
        ("Dec", 21.0),
    ];
    let data: Vec<Datum> = months.iter().map(|(m, v)| Datum::new(*m, *v)).collect();

    rsx! {
        ChartFrame { title: "Incidents", description: "Sev-2 and above, by month. Winter peaks with holiday traffic.",
            PolarAreaChart { data, label: "Incidents by month" }
        }
    }
}
