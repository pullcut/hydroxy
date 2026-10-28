use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::split_line_chart::{Annotation, Datum, SplitLineChart};

#[component]
pub fn SplitLineChartDemo() -> Element {
    // Monthly revenue in £ thousands: actual to July, then the plan.
    let revenue = vec![
        Datum::new("Jan", 118.0),
        Datum::new("Feb", 124.0),
        Datum::new("Mar", 131.0),
        Datum::new("Apr", 129.0),
        Datum::new("May", 140.0),
        Datum::new("Jun", 148.0),
        Datum::new("Jul", 155.0),
        Datum::new("Aug", 161.0),
        Datum::new("Sep", 168.0),
        Datum::new("Oct", 174.0),
        Datum::new("Nov", 181.0),
        Datum::new("Dec", 190.0),
    ];

    rsx! {
        ChartFrame { title: "Revenue", description: "Actual to July, then the plan for the rest of the year.",
            SplitLineChart {
                data: revenue,
                split: 7,
                label: "Monthly revenue, actual and projected",
                projected_label: "Plan",
                format: move |v: f64| format!("£{v:.0}k"),
                annotations: vec![Annotation::y_line(150.0, "Break-even")],
            }
        }
    }
}
