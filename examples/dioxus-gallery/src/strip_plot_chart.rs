use dioxus::prelude::*;
use hydroxy_dioxus::strip_plot_chart::{Annotation, Group, StripPlotChart};

#[component]
pub fn StripPlotChartDemo() -> Element {
    let groups = vec![
        Group::new("US East", [182.0, 195.0, 201.0, 188.0, 210.0, 224.0, 176.0, 199.0, 240.0, 205.0, 192.0, 186.0, 215.0, 230.0, 198.0, 310.0, 207.0, 189.0, 221.0, 203.0]),
        Group::new("EU West", [142.0, 151.0, 160.0, 138.0, 149.0, 172.0, 155.0, 147.0, 166.0, 158.0, 144.0, 139.0, 181.0, 152.0, 161.0, 148.0, 290.0, 157.0, 150.0, 163.0]),
        Group::new("Asia Pacific", [260.0, 284.0, 271.0, 299.0, 255.0, 318.0, 276.0, 290.0, 266.0, 305.0, 281.0, 340.0, 270.0, 262.0, 295.0, 287.0, 412.0, 279.0, 301.0, 268.0]),
    ];

    rsx! {
        StripPlotChart {
            groups,
            label: "API response time by region",
            annotations: vec![Annotation::x_line(300.0, "SLO")],
            format: move |v: f64| format!("{v:.0} ms"),
        }
    }
}
