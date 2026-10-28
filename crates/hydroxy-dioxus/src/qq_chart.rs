//! `QQChart`: a normal quantile–quantile plot. Each sorted sample value is plotted against
//! the normal quantile it would have if the sample were normal; points along the dashed
//! reference line mean the data is close to normal. Geometry comes from `hydroxy_charts::qq`.

use dioxus::prelude::*;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tick, Tip, TipRow, colour};
use hydroxy_charts::scale::Linear;
use hydroxy_charts::{format_value, qq};

pub use hydroxy_charts::annotate::Annotation;

use crate::plot_kit::{Plot, formatter};

/// Sample values (up the side) against normal quantiles in standard deviations (along the
/// bottom). Hover or touch a point to read it; focus the plot and use Left/Right to step
/// from smallest to largest, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn QQChart(
    /// The sample, in any order. NaN values are dropped.
    sample: ReadSignal<Vec<f64>>,
    /// What the chart shows, such as "Checkout latency residuals". Used as the accessible name.
    label: String,
    /// What the sample measures, for the tooltip and table.
    #[props(default = "Value".to_string())]
    value_label: String,
    /// Plot height in px, excluding the axis labels.
    #[props(default = 260)]
    height: u16,
    /// Formats sample values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` is a normal quantile (standard
    /// deviations), `y` a sample value.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let notes = annotations();
    let layout = qq::layout(&sample(), &notes);
    let n = layout.points.len();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let sx = Linear::new(layout.x_domain, (0.0, 1.0));

    let tip = current.map(|i| {
        let p = layout.points[i];
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(colour(0)), value_label.clone(), fmt(p.sample)) },
            TipRow::new(None, "Normal quantile", format!("{:.2}σ", p.theoretical)),
        ];
        Tip { x: p.x, y: p.y, title: format!("{} of {n}", i + 1), rows }
    });
    let labels: Vec<Tick> = layout.x_ticks.iter().map(|g| Tick { at: g.fraction, text: format!("{}σ", format_value(g.value)), baseline: false }).collect();
    let rows: Vec<Vec<String>> = layout.points.iter().enumerate().map(|(i, p)| vec![(i + 1).to_string(), format!("{:.2}", p.theoretical), fmt(p.sample)]).collect();
    let xs: Vec<f64> = layout.points.iter().map(|p| p.x).collect();

    rsx! {
        Plot {
            scope: "qq-chart",
            label,
            hint: "Use the left and right arrow keys to step from smallest to largest.",
            height,
            ticks: layout.axis.ticks(fmt),
            labels,
            annotations: plot::place(&notes, false, |x| sx.map(x), |v| layout.axis.scale.map(v)),
            tip,
            head: vec!["Rank".to_string(), "Normal quantile".to_string(), value_label.clone()],
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
                    path { "data-part": "reference", d: "{layout.reference}" }
                }
                div { "data-part": "points",
                    for (i , p) in layout.points.iter().enumerate() {
                        span {
                            key: "{i}",
                            "data-part": "dot",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--c: {p.x:.4}; --v: {p.y:.4}; --i: {i}; background: {colour(0)};",
                        }
                    }
                }
            },
        }
    }
}
