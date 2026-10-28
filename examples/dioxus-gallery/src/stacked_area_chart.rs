use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::stacked_area_chart::{Annotation, Curve, Series, StackOffset, StackedAreaChart};

#[component]
pub fn StackedAreaChartDemo() -> Element {
    let weeks: Vec<String> = [
        "Jan 6", "Jan 13", "Jan 20", "Jan 27", "Feb 3", "Feb 10", "Feb 17", "Feb 24", "Mar 3", "Mar 10", "Mar 17",
        "Mar 24",
    ]
    .map(String::from)
    .to_vec();
    // Weekly visits in thousands, bottom of the stack first.
    let traffic = vec![
        Series::new("Organic search", [42.0, 44.0, 47.0, 45.0, 49.0, 52.0, 55.0, 53.0, 57.0, 60.0, 62.0, 65.0]),
        Series::new("Paid search", [18.0, 20.0, 22.0, 25.0, 24.0, 21.0, 19.0, 23.0, 26.0, 28.0, 27.0, 30.0]),
        Series::new("Referral", [9.0, 9.0, 10.0, 12.0, 11.0, 10.0, 13.0, 14.0, 13.0, 15.0, 16.0, 15.0]),
        Series::new("Social", [6.0, 8.0, 7.0, 9.0, 14.0, 18.0, 12.0, 10.0, 9.0, 11.0, 10.0, 12.0]),
        Series::new("Email", [4.0, 12.0, 5.0, 4.0, 13.0, 5.0, 4.0, 14.0, 5.0, 4.0, 15.0, 6.0]),
    ];
    let thousands = |v: f64| format!("{v:.0}k");

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Visits by source", description: "Weekly visits, stacked from zero. The dashed line is the quarter's weekly goal.",
                StackedAreaChart {
                    series: traffic.clone(),
                    labels: weeks.clone(),
                    label: "Weekly visits by traffic source",
                    curve: Curve::Monotone,
                    format: thousands,
                    annotations: vec![Annotation::y_line(120.0, "Goal")],
                }
            }
            ChartFrame { title: "Share of visits", description: "The same data scaled to 100%, so each source's share reads directly.",
                StackedAreaChart { series: traffic.clone(), labels: weeks.clone(), label: "Share of weekly visits by source", offset: StackOffset::Expand, format: thousands }
            }
            ChartFrame { title: "Traffic streamgraph", description: "A wiggle baseline keeps each ribbon as flat as it can. Values are relative, so there is no axis.",
                StackedAreaChart {
                    series: traffic,
                    labels: weeks,
                    label: "Weekly visits by source, as a streamgraph",
                    offset: StackOffset::Wiggle,
                    curve: Curve::Monotone,
                    format: thousands,
                }
            }
        }
    }
}
