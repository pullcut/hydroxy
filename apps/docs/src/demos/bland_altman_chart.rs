use leptos::prelude::*;
use hydroxy_leptos::bland_altman_chart::*;
use hydroxy_leptos::chart_frame::ChartFrame;

pub fn demo() -> AnyView {
    // Blood glucose in mmol/L for 40 patients: the clinic's lab analyser and a finger-prick
    // meter. The meter reads a little high, and more so at high readings.
    let lab: Vec<f64> = (0..40).map(|i| 4.2 + f64::from((i * 37) % 97) / 10.0).collect();
    let meter: Vec<f64> = lab.iter().zip(0..).map(|(v, i)| v * 1.03 + 0.15 + f64::from((i * 53) % 17) / 20.0 - 0.4).collect();

    view! {
        <ChartFrame title="Glucose meter against the lab" description="Each patient's two readings: their mean along the bottom, meter minus lab up the side.">
            <BlandAltmanChart
                a=meter
                b=lab
                a_label="Meter"
                b_label="Lab"
                label="Bland-Altman plot of meter and lab glucose"
                show_ci=true
                format=|v: f64| format!("{v:.1}")
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bland-altman-chart",
    name: "Bland-Altman chart",
    group: "Points",
    summary: "Check whether two ways of measuring the same thing agree. Each pair is plotted as its mean against its difference, with the average bias and the 95% limits of agreement.",
    demo,
    leptos: include_str!("bland_altman_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bland_altman_chart.rs"),
    parts: &[
        &["BlandAltmanChart", "root", "figure with a screen-reader table of each pair, its mean and difference"],
        &["", "dot", "One per pair: mean across, difference (a − b) up"],
        &["", "limit", "The mean difference (data-kind=bias, solid) and the limits of agreement (lower, upper; dashed)"],
        &["", "limit-band", "95% confidence interval of each line (show_ci)"],
        &["", "limit-label", "The value of each line"],
        &["", "tooltip", "Difference, mean and both readings"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next pair"],
        &["Home / End", "First or last pair"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["a", "Vec<f64>", "Required. Readings from the first method"],
        &["b", "Vec<f64>", "Required. Readings from the second method, paired by position"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["a_label / b_label", "String", "Names of the two methods. Default \"Method A\" / \"Method B\""],
        &["show_ci", "bool", "Shade the 95% confidence interval of the bias and each limit"],
        &["height", "u16", "Plot height in px. Default 280"],
        &["format", "Fn(f64) -> String", "Formats readings and differences"],
        &["annotations", "Vec<Annotation>", "x is a mean, y a difference"],
    ],
};
