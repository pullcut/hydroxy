use dioxus::prelude::*;
use oxide_dioxus::bar_chart::{Annotation, BarChart, Datum, Highlight, Orientation};
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::segmented::{Segmented, SegmentedItem, SegmentedOption};

#[component]
pub fn BarChartDemo() -> Element {
    let weeks = vec![
        Datum::new("2 Sep", 6.0),
        Datum::new("9 Sep", 9.0),
        Datum::new("16 Sep", 7.0),
        Datum::new("23 Sep", 12.0),
        Datum::new("30 Sep", 10.0),
        Datum::new("7 Oct", 14.0),
        Datum::new("14 Oct", 11.0),
        Datum::new("21 Oct", 17.0),
    ];
    let sources = vec![
        Datum::new("Customer interviews", 18.0),
        Datum::new("Support tickets", 14.0),
        Datum::new("Sales call notes", 11.0),
        Datum::new("In-app survey comments", 8.0),
        Datum::new("App store reviews", 5.0),
    ];
    let mut month = use_signal(|| Some(String::from("oct")));
    let spend = if month().as_deref() == Some("sep") {
        vec![
            Datum::new("Research", 3400.0),
            Datum::new("Support", 2900.0),
            Datum::new("Sales", 2200.0),
            Datum::new("Product", 2500.0),
        ]
    } else {
        vec![
            Datum::new("Research", 4200.0),
            Datum::new("Support", 2600.0),
            Datum::new("Sales", 3100.0),
            Datum::new("Product", 1800.0),
        ]
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Interviews read", description: "Per week, since the start of September, against a goal of 15.",
                BarChart { data: weeks, label: "Interviews read per week", annotations: vec![Annotation::y_line(15.0, "Goal")] }
            }
            Segmented {
                label: "Month",
                items: vec![SegmentedOption::new("sep"), SegmentedOption::new("oct")],
                value: month(),
                on_value_change: move |v| month.set(Some(v)),
                SegmentedItem { value: "sep", label: "September" }
                SegmentedItem { value: "oct", label: "October" }
            }
            ChartFrame { title: "Monthly spend", description: "By team, switch the month to watch the bars glide.",
                BarChart {
                    data: spend,
                    label: "Monthly spend by team",
                    height: 160,
                    highlight: Highlight::Index(0),
                    show_values: true,
                    format: move |v: f64| {
                        let k = v / 1000.0;
                        if k.fract() == 0.0 { format!("£{k:.0}k") } else { format!("£{k:.1}k") }
                    },
                }
            }
            ChartFrame { title: "Interviews by source", description: "Horizontal bars keep long names readable.",
                BarChart {
                    data: sources,
                    label: "Interviews by source",
                    orientation: Orientation::Horizontal,
                    height: 200,
                    highlight: Highlight::Index(0),
                    show_values: true,
                    annotations: vec![Annotation::y_line(11.2, "Average")],
                }
            }
        }
    }
}
