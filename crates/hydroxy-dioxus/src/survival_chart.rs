//! `SurvivalChart`: Kaplan–Meier survival curves. Each group's curve steps down at every
//! event; ticks mark censored subjects; an optional band shows the Greenwood 95% interval
//! and a table under the axis counts who is still at risk. Geometry comes from
//! `hydroxy_charts::survival`.

use dioxus::prelude::*;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{Key, Tick, Tip, TipRow, colour};
use hydroxy_charts::survival::{self, Step, survival_at};

pub use hydroxy_charts::survival::{Observation, SurvivalGroup};

use crate::plot_kit::{Plot, formatter, pct};

/// A survival share as a whole percentage, such as "75%".
fn share(p: f64) -> String {
    format!("{:.0}%", p * 100.0)
}

/// The estimate and its interval in force at `t`.
fn estimate(steps: &[Step], t: f64) -> (f64, (f64, f64)) {
    survival_at(steps, t).map_or((1.0, (1.0, 1.0)), |s| (s.survival, s.ci))
}

/// The share still event-free over time, one step curve per group (up to six). Hover or touch
/// to read every group's estimate at that time; focus the plot and use Left/Right to step
/// through observed times, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SurvivalChart(
    /// The groups: `SurvivalGroup::new(name, [(time, event), …])`, where `event` is `false`
    /// for a censored subject.
    data: ReadSignal<Vec<SurvivalGroup>>,
    /// What the chart shows, such as "Time to churn by plan". Used as the accessible name.
    label: String,
    /// What time is measured in, for the tooltip and table, such as "Months".
    #[props(default = "Time".to_string())]
    time_label: String,
    /// Shade each curve's Greenwood 95% confidence interval.
    #[props(default)]
    show_ci: bool,
    /// Show the number at risk at each time tick under the axis.
    #[props(default)]
    risk_table: bool,
    /// Plot height in px, excluding the axis labels, legend and risk table.
    #[props(default = 260)]
    height: u16,
    /// Formats times for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = formatter(format);
    let layout = survival::layout(&data());
    let n = layout.times.len();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let positions: Vec<f64> = layout.times.iter().map(|&t| layout.x_axis.scale.map(t)).collect();

    let at = current.map(|i| (layout.times[i], positions[i]));
    let tip = at.map(|(t, fx)| {
        let rows = layout
            .curves
            .iter()
            .enumerate()
            .map(|(g, c)| {
                let (s, (lo, hi)) = estimate(&c.steps, t);
                let value = if show_ci { format!("{} ({}–{})", share(s), share(lo), share(hi)) } else { share(s) };
                TipRow { active: layout.curves.len() == 1, ..TipRow::new(Some(colour(g)), c.name.clone(), value) }
            })
            .collect();
        let y = layout.curves.iter().map(|c| estimate(&c.steps, t).0).fold(0.0, f64::max);
        Tip { x: fx, y, title: format!("{time_label} {}", fmt(t)), rows }
    });
    let legend: Vec<Key> = if layout.curves.len() > 1 {
        layout.curves.iter().enumerate().map(|(g, c)| Key::new(c.name.clone(), colour(g))).collect()
    } else {
        Vec::new()
    };
    let rows: Vec<Vec<String>> = layout
        .curves
        .iter()
        .flat_map(|c| {
            c.steps.iter().map(|s| {
                vec![
                    c.name.clone(),
                    fmt(s.time),
                    s.at_risk.to_string(),
                    s.events.to_string(),
                    s.censored.to_string(),
                    share(s.survival),
                    share(s.ci.0),
                    share(s.ci.1),
                ]
            })
        })
        .collect();
    let ticks: Vec<Tick> = layout
        .axis
        .gridlines
        .iter()
        .map(|g| Tick { at: g.fraction, text: share(g.value), baseline: g.value == 0.0 })
        .collect();

    let variant = if show_ci { "ci" } else { "" };

    rsx! {
        Plot {
            scope: "survival-chart",
            label,
            hint: "Use the left and right arrow keys to step through times.",
            height,
            variant,
            ticks,
            labels: layout.x_axis.ticks(fmt),
            annotations: Vec::new(),
            legend,
            tip,
            head: ["Group", time_label.as_str(), "At risk", "Events", "Censored", "Survival", "95% low", "95% high"].map(String::from).to_vec(),
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
                        if show_ci && !c.band.is_empty() {
                            path { "data-part": "band", d: "{c.band}", fill: colour(g) }
                        }
                        path { "data-part": "line", d: "{c.line}", stroke: colour(g) }
                    }
                }
                div { "data-part": "points", "aria-hidden": "true",
                    for (g , c) in layout.curves.iter().enumerate() {
                        for (x , y) in c.censors.iter().copied() {
                            span { "data-part": "censor", style: "--c: {x:.4}; --v: {y:.4}; background: {colour(g)};" }
                        }
                    }
                }
            },
            overlay: at.map(|(t, fx)| rsx! {
                div { "data-part": "crosshair", style: "--p: {pct(fx)};" }
                for (g , c) in layout.curves.iter().enumerate() {
                    span { "data-part": "dot", style: "--c: {fx:.4}; --v: {estimate(&c.steps, t).0:.4}; background: {colour(g)};" }
                }
            }),
        }
        if risk_table {
            div { "data-scope": "survival-chart", "data-part": "risk-table", "aria-hidden": "true",
                div { "data-part": "risk-title", "Number at risk" }
                for (g , (c , counts)) in layout.curves.iter().zip(&layout.risk).enumerate() {
                    div { "data-part": "risk-row",
                        span { "data-part": "risk-name", title: "{c.name}",
                            span { "data-part": "swatch", style: "--swatch: {colour(g)};" }
                            "{c.name}"
                        }
                        div { "data-part": "risk-counts",
                            for (tick , count) in layout.x_axis.gridlines.iter().zip(counts) {
                                span { "data-part": "risk-count", style: "--p: {pct(tick.fraction)};", "{count}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
