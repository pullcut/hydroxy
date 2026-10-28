use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::span_chart::{Annotation, Span, SpanChart};

#[component]
pub fn SpanChartDemo() -> Element {
    let roles: Vec<String> =
        ["Junior designer", "Product designer", "Senior designer", "Design lead", "Head of design"].map(String::from).to_vec();
    // Each band runs from the lowest to the highest offer; the marker is the median.
    let bands = vec![
        Span::new(32.0, 41.0).with_marker(36.5),
        Span::new(42.0, 58.0).with_marker(49.0),
        Span::new(55.0, 74.0).with_marker(63.0),
        Span::new(70.0, 92.0).with_marker(80.0),
        Span::new(88.0, 125.0).with_marker(104.0),
    ];
    let cities: Vec<String> = ["Reykjavík", "Edinburgh", "Lisbon", "Athens"].map(String::from).to_vec();
    let july = vec![Span::new(9.0, 14.0), Span::new(11.0, 19.0), Span::new(18.0, 28.0), Span::new(23.0, 33.0)];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Salary bands", description: "Offers made in 2025, thousands per year. The dot is the median offer.",
                SpanChart {
                    rows: roles,
                    data: bands,
                    label: "Salary band by design role",
                    marker_label: "Median",
                    format: move |v: f64| format!("£{v:.0}k"),
                }
            }
            ChartFrame { title: "July temperatures", description: "Typical overnight low to daytime high.",
                SpanChart {
                    rows: cities,
                    data: july,
                    label: "Typical July temperature range by city",
                    format: move |v: f64| format!("{v:.0}°C"),
                    annotations: vec![Annotation::y_line(25.0, "Beach weather")],
                }
            }
        }
    }
}
