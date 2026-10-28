use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::lollipop_chart::{Annotation, Datum, LollipopChart, Orientation, ScaleKind};

#[component]
pub fn LollipopChartDemo() -> Element {
    let nps = vec![
        Datum::new("Onboarding", 42.0),
        Datum::new("Billing", 18.0),
        Datum::new("Search", 31.0),
        Datum::new("Reporting", 55.0),
        Datum::new("Mobile app", 12.0),
        Datum::new("Integrations", 37.0),
    ];
    let downloads = vec![
        Datum::new("Core library", 412_000.0),
        Datum::new("Command line tool", 38_500.0),
        Datum::new("Editor plugin", 9_200.0),
        Datum::new("Migration helper", 1_150.0),
        Datum::new("Legacy adapter", 140.0),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Satisfaction by area", description: "Net promoter score from last month's survey.",
                LollipopChart { data: nps, label: "Net promoter score by product area", annotations: vec![Annotation::y_line(30.0, "Goal")] }
            }
            ChartFrame { title: "Monthly downloads", description: "Log scale, so the small packages stay visible.",
                LollipopChart {
                    data: downloads,
                    label: "Monthly downloads by package",
                    orientation: Orientation::Horizontal,
                    scale: ScaleKind::Log,
                    format: move |v: f64| if v >= 1000.0 { format!("{:.0}k", v / 1000.0) } else { format!("{v:.0}") },
                }
            }
        }
    }
}
