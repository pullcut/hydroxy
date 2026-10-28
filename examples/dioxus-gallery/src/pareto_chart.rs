use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::pareto_chart::{Datum, ParetoChart};

#[component]
pub fn ParetoChartDemo() -> Element {
    let causes = vec![
        Datum::new("Login problems", 84.0),
        Datum::new("Billing questions", 212.0),
        Datum::new("Slow exports", 47.0),
        Datum::new("Missing invoices", 129.0),
        Datum::new("Sync conflicts", 66.0),
        Datum::new("Permissions", 31.0),
        Datum::new("Other", 18.0),
    ];

    rsx! {
        ChartFrame { title: "Tickets by cause", description: "Two causes account for most of the volume. The line reads on the right-hand axis.",
            ParetoChart { data: causes, label: "Support tickets by cause", value_label: "Tickets" }
        }
    }
}
