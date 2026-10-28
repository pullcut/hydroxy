//! `SplitLineChart`: one line, solid for actual values and dashed for the projection
//! after a split index. Geometry comes from `hydroxy_charts::split::split_line`.

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

/// Actual values as a solid line, then a projection as a dashed one, joined at the last
/// actual point. Hover or touch to read a point; focus the plot and use Left/Right to step,
/// Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SplitLineChart(
    /// The values in x order, actual first. NaN leaves a gap.
    data: ReadSignal<Vec<Datum>>,
    /// Index of the first projected value. Past the end, the whole line is actual.
    split: usize,
    /// What the chart shows, such as "Monthly revenue". Used as the accessible name.
    label: String,
    /// Names the solid part in the legend, tooltip and table.
    #[props(default = "Actual".to_string())]
    actual_label: String,
    /// Names the dashed part in the legend, tooltip and table.
    #[props(default = "Projected".to_string())]
    projected_label: String,
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
    let layout = split::split_line(&values, split, scale, curve, &notes);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let kind = |i: usize| if i < split { actual_label.clone() } else { projected_label.clone() };

    let tip = current.map(|i| {
        let rows = vec![TipRow { active: true, ..TipRow::new(Some(colour(0)), kind(i), plot::cell(Some(items[i].value), fmt)) }];
        Tip { x: layout.xs[i], y: layout.points[i].unwrap_or(0.0), title: items[i].label.clone(), rows }
    });
    let names: Vec<String> = items.iter().map(|d| d.label.clone()).collect();
    let rows: Vec<Vec<String>> = items.iter().enumerate().map(|(i, d)| vec![d.label.clone(), plot::cell(Some(d.value), fmt), kind(i)]).collect();
    let legend = vec![Key::new(actual_label.clone(), colour(0)), Key { dashed: true, ..Key::new(projected_label.clone(), colour(0)) }];
    let xs = layout.xs.clone();

    rsx! {
        Plot {
            scope: "split-line-chart",
            label,
            hint: "Use the left and right arrow keys to step through.",
            height,
            ticks: layout.axis.ticks(fmt),
            labels: plot::point_ticks(&names, &layout.xs, 8),
            annotations: plot::place(&notes, false, |x| plot::index_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head: vec![String::new(), "Value".to_string(), "Kind".to_string()],
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, _)| nearest_index(&xs, x));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    path { "data-part": "line", d: "{layout.before}", stroke: colour(0) }
                    path { "data-part": "line", "data-dashed": "", d: "{layout.after}", stroke: colour(0) }
                }
            },
            overlay: current.and_then(|i| {
                let (x, y) = (layout.xs[i], layout.points[i]?);
                Some(rsx! {
                    div { "data-part": "crosshair", style: "--p: {pct(x)};" }
                    span { "data-part": "dot", style: "--c: {x:.4}; --v: {y:.4}; background: {colour(0)};" }
                })
            }),
        }
    }
}
