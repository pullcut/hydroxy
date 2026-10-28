//! `ThresholdChart`: one line coloured differently above and below a threshold.
//! Geometry comes from `hydroxy_charts::split::threshold`; the line and its fill are drawn
//! twice and clipped at the threshold.

use dioxus::prelude::*;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use hydroxy_charts::split;

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct};

/// A line read against a threshold: above it in the first chart colour, below it in the
/// second, with the gap to the threshold shaded. Hover or touch to read a point; focus the
/// plot and use Left/Right to step, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ThresholdChart(
    /// The values in x order. NaN leaves a gap.
    data: ReadSignal<Vec<Datum>>,
    /// The value the line is judged against.
    threshold: f64,
    /// What the chart shows, such as "Server error rate". Used as the accessible name.
    label: String,
    /// Names the threshold line, tooltip row and table column.
    #[props(default = "Threshold".to_string())]
    threshold_label: String,
    /// Legend name for values above the threshold.
    #[props(default = "Above".to_string())]
    above_label: String,
    /// Legend name for values below the threshold.
    #[props(default = "Below".to_string())]
    below_label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 240)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[props(default)]
    curve: Curve,
    /// Linear (default), Log or Symlog value axis.
    #[props(default)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a point index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let items = data();
    let n = items.len();
    let notes = annotations();
    let values: Vec<f64> = items.iter().map(|d| d.value).collect();
    let layout = split::threshold(&values, threshold, scale, curve, &notes);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let side = |v: f64| colour(usize::from(v < threshold));

    let tip = current.map(|i| {
        let d = &items[i];
        let gap = d.value - threshold;
        let sign = if gap >= 0.0 { "+" } else { "−" };
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(side(d.value)), "", plot::cell(Some(d.value), fmt)) },
            TipRow::new(None, threshold_label.clone(), fmt(threshold)),
            TipRow::new(
                None,
                "Difference",
                if d.value.is_finite() { format!("{sign}{}", fmt(gap.abs())) } else { "–".into() },
            ),
        ];
        Tip { x: layout.xs[i], y: layout.points[i].unwrap_or(layout.threshold), title: d.label.clone(), rows }
    });
    let names: Vec<String> = items.iter().map(|d| d.label.clone()).collect();
    let rows: Vec<Vec<String>> =
        items.iter().map(|d| vec![d.label.clone(), plot::cell(Some(d.value), fmt), fmt(threshold)]).collect();
    let t = layout.threshold;
    let xs = layout.xs.clone();

    rsx! {
        Plot {
            scope: "threshold-chart",
            label,
            hint: "Use the left and right arrow keys to step through.",
            height,
            ticks: layout.axis.ticks(fmt),
            labels: plot::point_ticks(&names, &layout.xs, 8),
            annotations: plot::place(&notes, false, |x| plot::index_at(x, n), |v| layout.axis.scale.map(v)),
            legend: vec![Key::new(above_label, colour(0)), Key::new(below_label, colour(1))],
            tip,
            head: vec![String::new(), "Value".to_string(), threshold_label.clone()],
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, _)| nearest_index(&xs, x));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                // The same drawing twice: once showing only what is above the threshold, once below.
                for (i , clip) in [format!("clip-path: inset(0 0 {} 0);", pct(t)), format!("clip-path: inset({} 0 0 0);", pct(1.0 - t))].into_iter().enumerate() {
                    div { key: "{i}", "data-part": "clip", "data-side": if i == 0 { "above" } else { "below" }, style: clip,
                        svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                            path { "data-part": "area", d: "{layout.area}", fill: colour(i) }
                            path { "data-part": "line", d: "{layout.line}", stroke: colour(i) }
                        }
                    }
                }
                div { "data-part": "threshold", style: "--p: {pct(t)};",
                    span { "{threshold_label}" }
                }
            },
            overlay: current.and_then(|i| {
                let (x, y) = (layout.xs[i], layout.points[i]?);
                Some(rsx! {
                    div { "data-part": "crosshair", style: "--p: {pct(x)};" }
                    span { "data-part": "dot", style: "--c: {x:.4}; --v: {y:.4}; background: {side(items[i].value)};" }
                })
            }),
        }
    }
}
