use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::pie_chart::{Datum, PieChart, PieLabels};

#[component]
pub fn PieChartDemo() -> Element {
    let regions = vec![
        Datum::new("North America", 412_000.0),
        Datum::new("Europe", 298_000.0),
        Datum::new("Asia Pacific", 187_000.0),
        Datum::new("Latin America", 64_000.0),
    ];
    let browsers = vec![
        Datum::new("Chrome", 6420.0),
        Datum::new("Safari", 2310.0),
        Datum::new("Edge", 940.0),
        Datum::new("Firefox", 610.0),
        Datum::new("Samsung Internet", 180.0),
        Datum::new("Opera", 95.0),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Revenue", description: "By region, last quarter.",
                PieChart { data: regions, label: "Revenue by region", format: move |v: f64| format!("${:.0}k", v / 1000.0) }
            }
            ChartFrame { title: "Sessions", description: "Leader lines keep small slices readable.",
                PieChart { data: browsers, label: "Sessions by browser", labels: PieLabels::Leader, size: 180 }
            }
        }
    }
}
