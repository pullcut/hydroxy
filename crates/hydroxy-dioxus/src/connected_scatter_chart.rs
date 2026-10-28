//! `ConnectedScatterChart`: two measures against each other, joined in time order.
//! Geometry comes from `hydroxy_charts::connected`.

use dioxus::prelude::*;
use hydroxy_charts::connected;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::connected::Moment;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct};

/// Plot width over height assumed for even timing and hit-testing; the plot stretches.
const ASPECT: f64 = 2.4;

/// How the pair of measures moved together over time: each moment is a dot at (x, y), and
/// a line joins them in time order, drawing itself from the first moment to the last. The
/// start is hollow, the end solid, and a few points carry their time. Hover or touch a dot
/// to read it; focus the plot and use Left/Right to step through time, Home/End to jump.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ConnectedScatterChart(
    /// The moments in time order: `Moment::new(label, x, y)`. NaN leaves a gap.
    data: ReadSignal<Vec<Moment>>,
    /// What the chart shows, such as "Unemployment against inflation, 2008–2024". Used as the accessible name.
    label: String,
    /// Names the horizontal measure, with its unit.
    x_label: String,
    /// Names the vertical measure, with its unit.
    y_label: String,
    /// Plot height in px, excluding the axis labels.
    #[props(default = 300)]
    height: u16,
    /// How many points carry a time label, at most. The first and last always do.
    #[props(default = 6)]
    time_labels: usize,
    /// Linear (default), Log or Symlog horizontal axis.
    #[props(default)]
    x_scale: ScaleKind,
    /// Linear (default), Log or Symlog vertical axis.
    #[props(default)]
    y_scale: ScaleKind,
    /// Formats horizontal values. Default: whole numbers, else one decimal place.
    format_x: Option<Callback<f64, String>>,
    /// Formats vertical values. Default: whole numbers, else one decimal place.
    format_y: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units on both axes.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let mask_id = use_hook(|| {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        format!("ox-connected-line-{}", NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    });
    let (fx, fy) = (formatter(format_x), formatter(format_y));
    let items = data();
    let n = items.len();
    let notes = annotations();
    let layout = connected::layout(&items, x_scale, y_scale, &notes, time_labels, ASPECT);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);

    let tip = current.and_then(|i| {
        let d = &items[i];
        let (x, y) = layout.points[i]?;
        let rows = vec![
            TipRow::new(None, x_label.clone(), plot::cell(Some(d.x), fx)),
            TipRow::new(None, y_label.clone(), plot::cell(Some(d.y), fy)),
        ];
        Some(Tip { x, y, title: d.label.clone(), rows })
    });
    let head = vec![String::new(), x_label.clone(), y_label.clone()];
    let rows: Vec<Vec<String>> =
        items.iter().map(|d| vec![d.label.clone(), plot::cell(Some(d.x), fx), plot::cell(Some(d.y), fy)]).collect();
    let first = layout.points.iter().position(Option::is_some);
    let last = layout.points.iter().rposition(Option::is_some);
    let dots: Vec<(usize, f64, f64, f64)> =
        layout.points.iter().enumerate().filter_map(|(i, p)| p.map(|(x, y)| (i, x, y, layout.along[i]))).collect();
    let times: Vec<(usize, f64, f64, f64, String)> = layout
        .labelled
        .iter()
        .filter_map(|&i| layout.points[i].map(|(x, y)| (i, x, y, layout.along[i], items[i].label.clone())))
        .collect();
    let points = layout.points.clone();

    rsx! {
        Plot {
            scope: "connected-scatter-chart",
            label,
            hint: "Use the left and right arrow keys to step through time.",
            height,
            ticks: layout.y_axis.ticks(fy),
            labels: layout.x_axis.ticks(fx),
            annotations: plot::place(&notes, false, |x| layout.x_axis.scale.map(x), |v| layout.y_axis.scale.map(v)),
            tip,
            head,
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|p| connected::nearest_point(&points, p, ASPECT, 0.12));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "x-grid", "aria-hidden": "true",
                    for (k , g) in layout.x_axis.gridlines.iter().enumerate() {
                        div { key: "{k}", "data-part": "x-gridline", style: "--p: {pct(g.fraction)};" }
                    }
                }
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    defs {
                        mask { id: "{mask_id}", "maskUnits": "userSpaceOnUse", x: "-10", y: "-10", width: "120", height: "120",
                            path { "data-part": "line-mask", d: "{layout.path}", "pathLength": "1" }
                        }
                    }
                    path { "data-part": "line", d: "{layout.path}", stroke: colour(0), mask: "url(#{mask_id})" }
                }
                div { "data-part": "points", "aria-hidden": "true",
                    for (i , x , y , t) in dots {
                        span {
                            key: "{i}",
                            "data-part": "dot",
                            "data-end": if Some(i) == last { Some("end") } else if Some(i) == first { Some("start") } else { None },
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--c: {x:.4}; --v: {y:.4}; --t: {t:.4};",
                        }
                    }
                    for (i , x , y , t , text) in times {
                        span {
                            key: "label-{i}",
                            "data-part": "point-label",
                            "data-side": if x > 0.85 { "start" } else { "end" },
                            style: "--c: {x:.4}; --v: {y:.4}; --t: {t:.4};",
                            "{text}"
                        }
                    }
                }
                span { "data-part": "axis-title", "data-axis": "y", "aria-hidden": "true", "{y_label}" }
                span { "data-part": "axis-title", "data-axis": "x", "aria-hidden": "true", "{x_label}" }
            },
        }
    }
}
