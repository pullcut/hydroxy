//! `ErrorBarChart`: an estimate per category with low and high whiskers.
//! Geometry comes from `hydroxy_charts::interval`.

use dioxus::prelude::*;
use hydroxy_charts::interval;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::Interval;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter};

/// Share of each category band a bar fills.
const BAR: f64 = 0.5;

/// Estimates with their uncertainty: a dot (or a bar from zero) per category and a capped
/// whisker from low to high. Hover or touch a category to read all three; focus the plot
/// and use Left/Right to step through, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ErrorBarChart(
    /// The estimates, in display order: `Interval::new(label, value, low, high)`.
    data: ReadSignal<Vec<Interval>>,
    /// What the chart shows, such as "Conversion rate by variant". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels.
    #[props(default = 240)]
    height: u16,
    /// Draw a bar from zero to each estimate instead of a dot.
    #[props(default)]
    bars: bool,
    /// Linear (default), Log or Symlog value axis.
    #[props(default)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a category index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let items = data();
    let n = items.len();
    let notes = annotations();
    let layout = interval::whiskers(&items, scale, bars, &notes);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let tip = current.map(|i| {
        let (d, w) = (&items[i], layout.whiskers[i]);
        let rows = vec![
            TipRow::new(Some(colour(0)), "Estimate", plot::cell(Some(d.value), fmt)),
            TipRow::new(None, "Low", plot::cell(Some(d.low), fmt)),
            TipRow::new(None, "High", plot::cell(Some(d.high), fmt)),
        ];
        let y = [w.value, w.low, w.high].into_iter().flatten().fold(0.0, f64::max);
        Tip { x: plot::band_centre(i, n), y, title: d.label.clone(), rows }
    });
    let names: Vec<String> = items.iter().map(|d| d.label.clone()).collect();
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|d| {
            vec![
                d.label.clone(),
                plot::cell(Some(d.value), fmt),
                plot::cell(Some(d.low), fmt),
                plot::cell(Some(d.high), fmt),
            ]
        })
        .collect();
    #[allow(clippy::cast_precision_loss, reason = "category counts are small")]
    let width = BAR / n.max(1) as f64;
    let base = layout.axis.base();
    let variant = if bars { "bars" } else { "points" };

    rsx! {
        Plot {
            scope: "error-bar-chart",
            label,
            hint: "Use the left and right arrow keys to step through.",
            height,
            variant,
            ticks: layout.axis.ticks(fmt),
            labels: plot::category_ticks(&names, 12),
            annotations: plot::place(&notes, false, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            tip,
            head: ["", "Estimate", "Low", "High"].map(String::from).to_vec(),
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
                    for (i , w) in layout.whiskers.iter().enumerate() {
                        if let Some(v) = w.value.filter(|_| bars) {
                            div {
                                "data-part": "bar",
                                "data-negative": (v < base).then_some(""),
                                "data-active": (current == Some(i)).then_some(""),
                                style: "--c: {plot::band_centre(i, n):.4}; --w: {width:.4}; --a: {base.min(v):.4}; --b: {base.max(v):.4}; --i: {i};",
                            }
                        }
                        if let Some((lo , hi)) = w.low.zip(w.high) {
                            span {
                                "data-part": "whisker",
                                style: "--c: {plot::band_centre(i, n):.4}; --w: 0; --a: {lo.min(hi):.4}; --b: {lo.max(hi):.4}; --i: {i};",
                            }
                        }
                        if let Some(v) = w.value.filter(|_| !bars) {
                            span {
                                "data-part": "dot",
                                "data-active": (current == Some(i)).then_some(""),
                                style: "--c: {plot::band_centre(i, n):.4}; --v: {v:.4}; --i: {i};",
                            }
                        }
                    }
                }
            },
        }
    }
}
