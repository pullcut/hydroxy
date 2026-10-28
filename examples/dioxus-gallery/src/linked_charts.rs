use dioxus::prelude::*;
use hydroxy_charts::event::View;
use hydroxy_dioxus::bar_chart::{BarChart, Datum, Highlight};
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::line_chart::{LineChart, Series};

const TICKETS: [(&str, &str); 12] = [
    ("Research", "Interview backlog"),
    ("Research", "Tag the September calls"),
    ("Research", "Survey wording"),
    ("Research", "Recruit five users"),
    ("Support", "Refund wording"),
    ("Support", "Macro cleanup"),
    ("Support", "Weekend cover"),
    ("Sales", "Pricing page copy"),
    ("Sales", "Trial follow-ups"),
    ("Sales", "Renewal list"),
    ("Product", "Export to CSV"),
    ("Product", "Dark mode contrast"),
];

#[component]
pub fn LinkedChartsDemo() -> Element {
    // Two line charts share one view: zoom or pan either and the other follows.
    let mut view = use_signal(|| View::FULL);
    let weeks: Vec<String> = (1..=24).map(|w| format!("Wk {w}")).collect();
    let signups: Vec<f64> =
        (0..24u32).map(|i| 300.0 + f64::from(i) * 9.0 + (f64::from(i) * 0.7).sin() * 30.0).collect();
    let churn: Vec<f64> = (0..24u32).map(|i| 40.0 - f64::from(i) * 0.4 + (f64::from(i) * 0.9).cos() * 5.0).collect();

    // Clicking a bar filters the table to that team's tickets.
    let teams = vec![
        Datum::new("Research", 4.0),
        Datum::new("Support", 3.0),
        Datum::new("Sales", 3.0),
        Datum::new("Product", 2.0),
    ];
    let mut picked = use_signal(|| None::<String>);
    let team = picked();

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Signups", description: "Scroll or press plus on the plot to zoom. The chart below follows.",
                LineChart {
                    series: vec![Series::new("Signups", signups)],
                    labels: weeks.clone(),
                    label: "Signups per week",
                    zoom: true,
                    view: Some(view()),
                    on_view_change: move |v| view.set(v),
                }
            }
            ChartFrame { title: "Cancellations", description: "Shares the view above.",
                LineChart {
                    series: vec![Series::new("Cancellations", churn)],
                    labels: weeks,
                    label: "Cancellations per week",
                    zoom: true,
                    view: Some(view()),
                    on_view_change: move |v| view.set(v),
                }
            }
            ChartFrame { title: "Open tickets", description: "Click a bar to list that team's tickets.",
                BarChart {
                    data: teams,
                    label: "Open tickets by team",
                    highlight: Highlight::Off,
                    on_select: move |d: hydroxy_charts::event::Datum| picked.set(d.label),
                }
            }
            div { class: "table-wrap",
                table {
                    caption { {team.clone().map_or_else(|| "All teams".to_owned(), |t| format!("{t} tickets"))} }
                    tbody {
                        for (t, title) in TICKETS.iter().filter(|(t, _)| team.as_deref().is_none_or(|p| p == *t)) {
                            tr { td { "{t}" } td { "{title}" } }
                        }
                    }
                }
            }
        }
    }
}
