use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::heatmap_chart::HeatmapChart;

#[component]
pub fn HeatmapChartDemo() -> Element {
    let days: Vec<String> = ["Mon", "Tue", "Wed", "Thu", "Fri"].map(String::from).to_vec();
    let hours: Vec<String> = ["9", "10", "11", "12", "13", "14", "15", "16"].map(String::from).to_vec();
    let sessions = vec![
        vec![12.0, 24.0, 31.0, 18.0, 9.0, 22.0, 27.0, 14.0],
        vec![15.0, 29.0, 36.0, 21.0, 11.0, 26.0, 33.0, 17.0],
        vec![14.0, 27.0, 40.0, 25.0, 13.0, 30.0, 35.0, 19.0],
        vec![11.0, 23.0, 33.0, 19.0, f64::NAN, 24.0, 29.0, 13.0],
        vec![8.0, 16.0, 21.0, 12.0, 6.0, 10.0, 12.0, 5.0],
    ];

    rsx! {
        ChartFrame {
            title: "Sessions by weekday and hour",
            description: "Darker cells are busier. Hover a cell, or focus the grid and use the arrow keys.",
            HeatmapChart {
                values: sessions,
                rows: days,
                columns: hours,
                label: "Sessions by weekday and hour of day",
                show_values: true,
            }
        }
    }
}
