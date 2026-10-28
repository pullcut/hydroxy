use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::connected_scatter_chart::{Annotation, ConnectedScatterChart, Moment};

#[component]
pub fn ConnectedScatterChartDemo() -> Element {
    // Annual averages: unemployment rate and consumer price inflation, both in percent.
    let years = [
        (2008, 5.7, 3.6), (2009, 7.6, 2.2), (2010, 7.9, 3.3), (2011, 8.1, 4.5), (2012, 8.0, 2.8), (2013, 7.6, 2.6),
        (2014, 6.2, 1.5), (2015, 5.4, 0.0), (2016, 4.9, 0.7), (2017, 4.4, 2.7), (2018, 4.1, 2.5), (2019, 3.8, 1.8),
        (2020, 4.6, 0.9), (2021, 4.5, 2.6), (2022, 3.9, 9.1), (2023, 4.0, 7.3), (2024, 4.3, 2.5),
    ];
    let data: Vec<Moment> = years.iter().map(|&(y, u, i)| Moment::new(y.to_string(), u, i)).collect();

    rsx! {
        ChartFrame { title: "Unemployment and inflation", description: "Each dot is one year, joined in order from 2008 (hollow) to 2024 (solid).",
            ConnectedScatterChart {
                data,
                label: "Unemployment against inflation, 2008 to 2024",
                x_label: "Unemployment rate",
                y_label: "Inflation",
                format_x: move |v: f64| format!("{v:.1}%"),
                format_y: move |v: f64| format!("{v:.1}%"),
                annotations: vec![Annotation::y_line(2.0, "2% target")],
            }
        }
    }
}
