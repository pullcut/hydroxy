use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::forest_plot_chart::*;
use leptos::prelude::*;

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

pub fn demo() -> AnyView {
    let (studies, pooled) = trials();
    view! {
        <ChartFrame title="Stroke within five years" description="Odds ratios from seven placebo-controlled trials, pooled by inverse-variance weight.">
            <ForestPlotChart
                studies
                pooled=Some(pooled)
                label="Odds of a stroke within five years, treatment against placebo"
                measure="Odds ratio"
                log=true
                favours=("Favours treatment".to_string(), "Favours placebo".to_string())
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "forest-plot-chart",
    name: "Forest plot",
    group: "Distributions",
    summary: "A meta-analysis at a glance: each study's estimate as a square sized by weight with its confidence interval, a line of no effect, the pooled estimate as a diamond, and the numbers in columns alongside. Ratios read on a log axis.",
    demo,
    leptos: include_str!("forest_plot_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/forest_plot_chart.rs"),
    parts: &[
        &["ForestPlotChart", "root", "figure; `data-active` while a row is picked out"],
        &["", "plot", "Focusable grid of rows; arrow keys step through the studies, then the pooled row"],
        &["", "head", "Column headings"],
        &["", "row", "One study, or the pooled result with `data-pooled`; `data-active` when picked out"],
        &["", "study", "The study's name"],
        &["", "interval", "The plot cell holding the marks"],
        &["", "square", "The estimate; its area tracks the study's weight"],
        &["", "whisker", "The confidence interval; extends out from the estimate"],
        &["", "diamond", "The pooled estimate and interval; lands last"],
        &["", "estimate", "Estimate and interval as text"],
        &["", "weight", "The study's share of the total weight"],
        &["", "null", "The line of no effect: 1 on a log axis, else 0"],
        &["", "axis", "Ticks under the plot column, with the favours labels either side of no effect"],
        &["", "tooltip", "Estimate, interval and weight of the active row"],
        &["", "sr-table", "Every row with its estimate, interval and weight"],
    ],
    keys: &[
        &["Arrow keys", "Step through the rows"],
        &["Home / End", "First or last row"],
        &["Escape", "Clear the active row"],
    ],
    props: &[
        &["studies", "Vec<Study>", "Required. Study::new(label, estimate, (low, high), weight)"],
        &["pooled", "Option<Study>", "The pooled result, drawn as a diamond; its weight is ignored"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["measure", "String", "Required. The effect measure, such as \"Odds ratio\""],
        &["log", "bool", "Log axis with no effect at 1, for ratios. Default false (no effect at 0)"],
        &["favours", "(String, String)", "What each side of no effect means, shown under the axis"],
        &["level", "u8", "Confidence level in the column heading. Default 95"],
        &["format", "Fn(f64) -> String", "Formats estimates, limits and ticks. Default: two decimal places"],
    ],
};
