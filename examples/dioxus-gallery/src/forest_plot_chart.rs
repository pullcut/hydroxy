use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::forest_plot_chart::{ForestPlotChart, Study};

/// Seven trials of a blood pressure drug against placebo: odds ratio of a stroke within
/// five years, its 95% interval, and the trial's inverse-variance weight.
fn trials() -> (Vec<Study>, Study) {
    let studies = vec![
        Study::new("Alvarez 2011", 0.72, (0.55, 0.94), 18.2),
        Study::new("Brennan 2013", 0.85, (0.62, 1.17), 12.5),
        Study::new("Chowdhury 2014", 0.64, (0.45, 0.91), 10.1),
        Study::new("Duarte 2016", 0.91, (0.74, 1.12), 24.3),
        Study::new("Eriksen 2018", 0.58, (0.36, 0.93), 6.4),
        Study::new("Fujimoto 2019", 0.79, (0.63, 0.99), 20.6),
        Study::new("Gallagher 2021", 1.08, (0.71, 1.64), 7.9),
    ];
    (studies, Study::new("Pooled (fixed effect)", 0.80, (0.72, 0.89), 100.0))
}

#[component]
pub fn ForestPlotChartDemo() -> Element {
    let (studies, pooled) = trials();
    rsx! {
        ChartFrame { title: "Stroke within five years", description: "Odds ratios from seven placebo-controlled trials, pooled by inverse-variance weight.",
            ForestPlotChart {
                studies,
                pooled: Some(pooled),
                label: "Odds of a stroke within five years, treatment against placebo",
                measure: "Odds ratio",
                log: true,
                favours: ("Favours treatment".to_string(), "Favours placebo".to_string()),
            }
        }
    }
}
