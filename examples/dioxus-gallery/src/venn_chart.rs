use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::venn_chart::{Intersection, VennChart};

#[component]
pub fn VennChartDemo() -> Element {
    // Customers by the exact mix of places they bought from this year.
    let channels = vec![
        Intersection::new(["Web store"], 4120.0),
        Intersection::new(["Mobile app"], 2630.0),
        Intersection::new(["Retail stores"], 3380.0),
        Intersection::new(["Web store", "Mobile app"], 1840.0),
        Intersection::new(["Web store", "Retail stores"], 960.0),
        Intersection::new(["Mobile app", "Retail stores"], 410.0),
        Intersection::new(["Web store", "Mobile app", "Retail stores"], 520.0),
    ];

    rsx! {
        ChartFrame {
            title: "Where customers buy",
            description: "Circle areas match each channel's customers, and each overlap matches the customers two channels share.",
            VennChart { data: channels, label: "Customers by purchase channel" }
        }
    }
}
