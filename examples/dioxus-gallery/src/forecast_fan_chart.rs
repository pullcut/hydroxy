use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::forecast_fan_chart::{Datum, FanBand, ForecastFanChart};

#[component]
pub fn ForecastFanChartDemo() -> Element {
    let months = |names: &[&str], values: &[f64]| names.iter().zip(values).map(|(n, &v)| Datum::new(*n, v)).collect::<Vec<_>>();
    let history = months(
        &["Nov 25", "Dec 25", "Jan 26", "Feb 26", "Mar 26", "Apr 26", "May 26", "Jun 26", "Jul 26", "Aug 26", "Sep 26", "Oct 26"],
        &[3120.0, 3240.0, 2980.0, 3310.0, 3450.0, 3390.0, 3560.0, 3620.0, 3480.0, 3710.0, 3840.0, 3920.0],
    );
    let forecast = months(&["Nov 26", "Dec 26", "Jan 27", "Feb 27", "Mar 27", "Apr 27"], &[4010.0, 4080.0, 4150.0, 4230.0, 4300.0, 4380.0]);
    let around = |half: [f64; 6]| {
        let mid = [4010.0, 4080.0, 4150.0, 4230.0, 4300.0, 4380.0];
        (mid.iter().zip(half).map(|(m, h)| m - h).collect::<Vec<_>>(), mid.iter().zip(half).map(|(m, h)| m + h).collect::<Vec<_>>())
    };
    let (low95, high95) = around([190.0, 290.0, 380.0, 460.0, 530.0, 610.0]);
    let (low80, high80) = around([120.0, 190.0, 250.0, 300.0, 350.0, 400.0]);
    // Widest first, so the narrower band draws on top.
    let bands = vec![FanBand::new("95% interval", low95, high95), FanBand::new("80% interval", low80, high80)];

    rsx! {
        ChartFrame { title: "Monthly orders", description: "Twelve months of orders and a six-month forecast with 80% and 95% intervals.",
            ForecastFanChart {
                history,
                forecast,
                bands,
                label: "Monthly orders with a six-month forecast",
                format: move |v: f64| format!("{:.1}k", v / 1000.0),
            }
        }
    }
}
