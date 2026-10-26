use dioxus::prelude::*;
use oxide_dioxus::line_chart::{AreaChart, Curve, LineChart, Series};

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

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            LineChart { series: users, labels: weeks, label: "Weekly active users" }
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
