use dioxus::prelude::*;
use oxide_dioxus::bar_chart::{BarChart, Datum};
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::segmented::{Segmented, SegmentedItem, SegmentedOption};

#[component]
pub fn ChartFrameDemo() -> Element {
    let weeks = vec![
        Datum::new("2 Sep", 6.0),
        Datum::new("9 Sep", 9.0),
        Datum::new("16 Sep", 7.0),
        Datum::new("23 Sep", 12.0),
        Datum::new("30 Sep", 10.0),
        Datum::new("7 Oct", 14.0),
    ];
    let mut mode = use_signal(|| Some(String::from("ready")));

    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            Segmented {
                label: "State",
                items: vec![SegmentedOption::new("ready"), SegmentedOption::new("loading"), SegmentedOption::new("empty")],
                value: mode(),
                on_value_change: move |v| mode.set(Some(v)),
                SegmentedItem { value: "ready", label: "Ready" }
                SegmentedItem { value: "loading", label: "Loading" }
                SegmentedItem { value: "empty", label: "Empty" }
            }
            ChartFrame {
                title: "Interviews read",
                description: "Per week, since the start of September.",
                loading: mode().as_deref() == Some("loading"),
                empty: mode().as_deref() == Some("empty"),
                empty_text: "No interviews in this period",
                BarChart { data: weeks, label: "Interviews read per week" }
            }
        }
    }
}
