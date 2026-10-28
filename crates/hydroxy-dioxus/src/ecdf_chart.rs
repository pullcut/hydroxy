//! `EcdfChart`: empirical cumulative distributions. Each group's step line rises by 1/n at
//! every value, so the height at any x is the share of values at or below it. Geometry comes
//! from `hydroxy_charts::ecdf`.

use dioxus::prelude::*;
use hydroxy_charts::ecdf::{self, ecdf_at};
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tick, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::ecdf::Sample;

use crate::plot_kit::{Plot, formatter, pct};

/// A cumulative share as a whole percentage, such as "72%".
fn share(p: f64) -> String {
    format!("{:.0}%", p * 100.0)
}

/// Step lines of the share of values at or below each x, one per group (up to six), with an
/// optional dashed normal fit for each. Hover or touch to read every group's percentile at
/// that value; focus the plot and use Left/Right to step through values, Home/End to jump,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn EcdfChart(
    /// The groups: `Sample::new(name, values)`. Up to six; NaN values are dropped.
    data: ReadSignal<Vec<Sample>>,
    /// What the chart shows, such as "Page load time by region". Used as the accessible name.
    label: String,
    /// Draw each group's fitted normal CDF as a dashed line, to judge normality.
    #[props(default)]
    reference: bool,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 260)]
    height: u16,
    /// Formats values for the x axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` is a value, `y` a cumulative share (0..=1).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let notes = annotations();
    let layout = ecdf::layout(&data(), &notes);
    let n = layout.xs.len();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let positions: Vec<f64> = layout.xs.iter().map(|&x| layout.x_axis.scale.map(x)).collect();

    let at = current.map(|i| (layout.xs[i], positions[i]));
    let tip = at.map(|(x, fx)| {
        let rows = layout
            .curves
            .iter()
            .enumerate()
            .map(|(g, c)| TipRow { active: layout.curves.len() == 1, ..TipRow::new(Some(colour(g)), c.name.clone(), share(ecdf_at(&c.sorted, x))) })
            .collect();
        let y = layout.curves.iter().map(|c| ecdf_at(&c.sorted, x)).fold(0.0, f64::max);
        Tip { x: fx, y, title: format!("At or below {}", fmt(x)), rows }
    });
    let mut legend: Vec<Key> = layout.curves.iter().enumerate().map(|(g, c)| Key::new(c.name.clone(), colour(g))).collect();
    if reference {
        legend.push(Key { dashed: true, ..Key::new("Normal fit", "var(--text-3)") });
    }
    if legend.len() == 1 {
        legend.clear();
    }
    let rows: Vec<Vec<String>> = layout
        .curves
        .iter()
        .map(|c| {
            let q = |p: f64| {
                // The smallest value whose ECDF reaches p.
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "sample sizes are small")]
                let k = ((p * c.sorted.len() as f64).ceil() as usize).clamp(1, c.sorted.len().max(1)) - 1;
                plot::cell(c.sorted.get(k).copied(), fmt)
            };
            vec![c.name.clone(), c.sorted.len().to_string(), fmt(c.mean), fmt(c.sd), q(0.1), q(0.25), q(0.5), q(0.75), q(0.9)]
        })
        .collect();
    let ticks: Vec<Tick> = layout.axis.gridlines.iter().map(|g| Tick { at: g.fraction, text: share(g.value), baseline: g.value == 0.0 }).collect();

    rsx! {
        Plot {
            scope: "ecdf-chart",
            label,
            hint: "Use the left and right arrow keys to step through values.",
            height,
            ticks,
            labels: layout.x_axis.ticks(fmt),
            annotations: plot::place(&notes, false, |x| layout.x_axis.scale.map(x), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head: ["Group", "n", "Mean", "SD", "10th percentile", "25th percentile", "Median", "75th percentile", "90th percentile"].map(String::from).to_vec(),
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, _)| nearest_index(&positions, x));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    for (g , c) in layout.curves.iter().enumerate() {
                        if reference && !c.reference.is_empty() {
                            path { "data-part": "reference", d: "{c.reference}", style: "stroke: {colour(g)};" }
                        }
                        path { "data-part": "line", d: "{c.line}", stroke: colour(g), style: "--i: {g};" }
                    }
                }
            },
            overlay: at.map(|(x, fx)| rsx! {
                div { "data-part": "crosshair", style: "--p: {pct(fx)};" }
                for (g , c) in layout.curves.iter().enumerate().filter(|(_, c)| !c.sorted.is_empty()) {
                    span { "data-part": "dot", style: "--c: {fx:.4}; --v: {ecdf_at(&c.sorted, x):.4}; background: {colour(g)};" }
                }
            }),
        }
    }
}
