//! `ParetoChart`: categories sorted largest first, with the running share of the total.
//! Geometry comes from `oxide_charts::waterfall::pareto`.
//!
//! This is the one cartesian chart with a second axis: the cumulative line is read
//! against a labelled 0–100% scale on the right, since a share and a count cannot share
//! one axis honestly.

use dioxus::prelude::*;
use oxide_charts::nav::list_key;
use oxide_charts::plot::{self, Key, Tick, Tip, TipRow, colour};
use oxide_charts::waterfall;

pub use oxide_charts::Datum;
pub use oxide_charts::annotate::Annotation;

use crate::plot_kit::{Plot, formatter};

/// Share of each category band a bar fills.
const BAR: f64 = 0.64;

fn share(f: f64) -> String {
    format!("{:.0}%", f * 100.0)
}

/// Bars sorted largest first, plus a line of their cumulative share of the total, read on a
/// 0–100% axis on the right (the one accepted exception to the single-axis rule). Hover or
/// touch a category to read both; focus the plot and use Left/Right to step through,
/// Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ParetoChart(
    /// The values, in any order; the chart sorts them. Negative or NaN values count as zero.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Support tickets by cause". Used as the accessible name.
    label: String,
    /// What the bars count, for the legend, tooltip and table.
    #[props(default = "Count".to_string())]
    value_label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 240)]
    height: u16,
    /// Formats values for the left axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, on the left axis (x is a position in sorted order).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let items = data();
    let notes = annotations();
    let values: Vec<f64> = items.iter().map(|d| d.value).collect();
    let layout = waterfall::pareto(&values, &notes);
    let sorted: Vec<Datum> = layout.order.iter().map(|&i| items[i].clone()).collect();
    let n = sorted.len();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let tip = current.map(|i| {
        let (bar, cum) = (layout.bars[i], layout.cumulative[i]);
        let rows = vec![TipRow::new(Some(colour(0)), value_label.clone(), fmt(sorted[i].value)), TipRow::new(Some(colour(1)), "Cumulative", share(cum))];
        Tip { x: plot::band_centre(i, n), y: bar.max(cum), title: sorted[i].label.clone(), rows }
    });
    let names: Vec<String> = sorted.iter().map(|d| d.label.clone()).collect();
    let rows: Vec<Vec<String>> = sorted.iter().zip(&layout.cumulative).map(|(d, &c)| vec![d.label.clone(), fmt(d.value), share(c)]).collect();
    #[allow(clippy::cast_precision_loss, reason = "category counts are small")]
    let width = BAR / n.max(1) as f64;
    let base = layout.axis.base();

    rsx! {
        Plot {
            scope: "pareto-chart",
            label,
            hint: "Use the left and right arrow keys to step through.",
            height,
            ticks: layout.axis.ticks(fmt),
            right_ticks: [0.0, 0.25, 0.5, 0.75, 1.0].map(|at| Tick { at, text: share(at), baseline: false }).to_vec(),
            labels: plot::category_ticks(&names, 12),
            annotations: plot::place(&notes, false, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            legend: vec![Key::new(value_label.clone(), colour(0)), Key::new("Cumulative share", colour(1))],
            tip,
            head: vec![String::new(), value_label.clone(), "Cumulative %".to_string()],
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, _)| plot::band_at(x, n));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "bars",
                    for (i , h) in layout.bars.iter().enumerate() {
                        div {
                            key: "{i}",
                            "data-part": "bar",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--c: {plot::band_centre(i, n):.4}; --w: {width:.4}; --a: {base:.4}; --b: {h:.4}; --i: {i}; background: {colour(0)};",
                        }
                    }
                }
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    path { "data-part": "line", d: "{layout.line}", stroke: colour(1) }
                }
                div { "data-part": "points",
                    for (i , c) in layout.cumulative.iter().enumerate() {
                        span {
                            key: "{i}",
                            "data-part": "dot",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--c: {plot::band_centre(i, n):.4}; --v: {c:.4}; --i: {i}; background: {colour(1)};",
                        }
                    }
                }
            },
        }
    }
}
