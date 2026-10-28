//! `CyclePlotChart`: one band per period of a cycle (month, weekday), each holding a
//! mini line across the cycles (years, weeks) and a flat line at that period's mean.
//! Geometry comes from `hydroxy_charts::cycle`.

use dioxus::prelude::*;
use hydroxy_charts::cycle;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct};

/// Seasonal shape and trend at once: the x axis is the periods of a cycle, and inside each
/// period a short line runs through that period's value in every cycle, oldest first, with
/// a flat line at their mean. Hover or touch to read a period; focus the plot and use
/// Left/Right to step through every point, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn CyclePlotChart(
    /// The periods along the axis, such as the months of the year.
    periods: ReadSignal<Vec<String>>,
    /// One series per cycle (such as a year), oldest first, with a value per period. NaN leaves a gap.
    cycles: ReadSignal<Vec<Series>>,
    /// What the chart shows, such as "Rainfall by month, 2021 to 2025". Used as the accessible name.
    label: String,
    /// Plot height in px.
    #[props(default = 260)]
    height: u16,
    /// Linear (default), Log or Symlog value axis. The axis fits the data rather than starting at zero.
    #[props(default)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a period index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let names = periods();
    let all = cycles();
    let notes = annotations();
    let (n, m) = (names.len(), all.len());
    let layout = cycle::layout(&all, n, scale, &notes);
    // Flattened as period * cycles + cycle, so Left/Right walk every point in reading order.
    let mut active = use_signal(|| None::<usize>);
    let split = move |i: usize| (i / m.max(1), i % m.max(1));
    let current = active().filter(|&i| i < n * m);
    let tip = current.map(split).and_then(|(p, c)| {
        let (x, y) = layout.points[p][c]?;
        let mean = layout.means[p];
        let rows = all
            .iter()
            .enumerate()
            .map(|(k, s)| TipRow { active: k == c, ..TipRow::new(None, s.name.clone(), plot::cell(s.values.get(p).copied(), fmt)) })
            .chain(mean.map(|mn| TipRow::new(Some("var(--text-2)".into()), "Mean", fmt(mn.value))))
            .collect();
        Some(Tip { x, y, title: names[p].clone(), rows })
    });
    let span = match (all.first(), all.last()) {
        (Some(a), Some(b)) if m > 1 => format!("{} to {}", a.name, b.name),
        (Some(a), _) => a.name.clone(),
        _ => String::new(),
    };
    let legend = vec![Key::new(span, colour(0)), Key::new("Mean", "var(--text-2)")];
    let head: Vec<String> = std::iter::once(String::new()).chain(all.iter().map(|s| s.name.clone())).chain(["Mean".to_string()]).collect();
    let rows: Vec<Vec<String>> = names
        .iter()
        .enumerate()
        .map(|(p, name)| {
            std::iter::once(name.clone())
                .chain(all.iter().map(|s| plot::cell(s.values.get(p).copied(), fmt)))
                .chain([plot::cell(layout.means[p].map(|mn| mn.value), fmt)])
                .collect()
        })
        .collect();
    let latest: Vec<(usize, (f64, f64))> = layout.points.iter().enumerate().filter_map(|(p, row)| row.last().copied().flatten().map(|pt| (p, pt))).collect();
    let active_dot = current.map(split).and_then(|(p, c)| layout.points[p][c]);
    let for_pointer = layout.clone();

    rsx! {
        Plot {
            scope: "cycle-plot-chart",
            label,
            hint: "Use the left and right arrow keys to step through every point.",
            height,
            ticks: layout.axis.ticks(fmt),
            labels: plot::category_ticks(&names, 12),
            annotations: plot::place(&notes, false, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head,
            rows,
            on_key: move |key: String| list_key(&key, active(), n * m).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| cycle::nearest(&for_pointer, x, y)).map(|(p, c)| p * m + c);
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    path { "data-part": "line", d: "{layout.lines}", stroke: colour(0) }
                }
                div { "data-part": "points",
                    for (p , mn) in layout.means.iter().enumerate().filter_map(|(p, mn)| mn.map(|mn| (p, mn))) {
                        span { "data-part": "mean", style: "left: {pct(mn.from)}; width: {pct(mn.to - mn.from)}; bottom: {pct(mn.at)}; --i: {p};" }
                    }
                    for (p , (x , y)) in latest {
                        span { "data-part": "dot", style: "--c: {x:.4}; --v: {y:.4}; --i: {p}; background: {colour(0)};" }
                    }
                }
            },
            overlay: active_dot.map(|(x, y)| rsx! {
                span { "data-part": "dot", style: "--c: {x:.4}; --v: {y:.4}; background: {colour(0)};" }
            }),
        }
    }
}
