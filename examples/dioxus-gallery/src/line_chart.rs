use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::line_chart::{AreaChart, Band, Curve, LineChart, Series};

#[component]
pub fn LineChartDemo() -> Element {
    let weeks: Vec<String> = ["5 Aug", "12 Aug", "19 Aug", "26 Aug", "2 Sep", "9 Sep", "16 Sep", "23 Sep", "30 Sep", "7 Oct", "14 Oct", "21 Oct"]
        .map(String::from)
        .to_vec();
    let users = vec![
        Series::new("This year", [412.0, 438.0, 431.0, 470.0, 502.0, 495.0, 540.0, 566.0, 551.0, 598.0, 627.0, 654.0]),
        Series::new("Last year", [380.0, 371.0, 395.0, 402.0, 398.0, 421.0, 433.0, 429.0, 447.0, 452.0, 470.0, 468.0]),
    ];
    let months: Vec<String> = ["May", "Jun", "Jul", "Aug", "Sep", "Oct"].map(String::from).to_vec();
    let revenue = vec![Series::new("Revenue", [18_400.0, 21_900.0, f64::NAN, 24_300.0, 27_800.0, 31_200.0])];

    let days: Vec<String> = (0..48).map(|i| format!("Day {}", i + 1)).collect();
    let forecast: Vec<f64> = (0..48u32).map(|i| 200.0 + f64::from(i) * 3.0 + (f64::from(i) * 0.5).sin() * 14.0).collect();
    // The interval widens the further out the forecast goes.
    let spread = |i: u32| 8.0 + f64::from(i) * 0.9;
    let band = Band::new(
        0,
        forecast.iter().zip(0..).map(|(f, i)| f - spread(i)).collect::<Vec<_>>(),
        forecast.iter().zip(0..).map(|(f, i)| f + spread(i)).collect::<Vec<_>>(),
    );
    let forecast = vec![Series::new("Forecast", forecast)];

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame {
                title: "Forecast",
                description: "The shaded band is the 90% interval. Drag the strip below, or focus an edge and use the arrow keys, to zoom.",
                LineChart { series: forecast, labels: days, label: "Forecast", curve: Curve::Monotone, bands: vec![band], brush: true }
            }
            ChartFrame { title: "Weekly active users", description: "This year against last year.",
                LineChart { series: users, labels: weeks, label: "Weekly active users" }
            }
            ChartFrame { title: "Monthly revenue", description: "July is missing, so the line breaks there.",
                AreaChart {
                    series: revenue,
                    labels: months,
                    label: "Monthly revenue",
                    height: 180,
                    curve: Curve::Monotone,
                    format: move |v: f64| format!("£{:.0}k", v / 1000.0),
                }
            }
        }
    }
}
