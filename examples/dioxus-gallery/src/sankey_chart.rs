use dioxus::prelude::*;
use oxide_dioxus::sankey_chart::{Link, SankeyChart};

#[component]
pub fn SankeyChartDemo() -> Element {
    let flows = vec![
        Link::new("Organic", "Visits", 4200.0),
        Link::new("Paid", "Visits", 1800.0),
        Link::new("Referral", "Visits", 900.0),
        Link::new("Visits", "Signed up", 2100.0),
        Link::new("Visits", "Left", 4800.0),
        Link::new("Signed up", "Free", 1300.0),
        Link::new("Signed up", "Team", 600.0),
        Link::new("Signed up", "Business", 200.0),
        Link::new("Free", "Team", 150.0),
        Link::new("Free", "Churned", 700.0),
        Link::new("Team", "Retained", 640.0),
        Link::new("Team", "Churned", 110.0),
        Link::new("Business", "Retained", 190.0),
    ];

    rsx! {
        SankeyChart { links: flows, label: "Visits to plans" }
    }
}
