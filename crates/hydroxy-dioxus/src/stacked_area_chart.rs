//! `StackedAreaChart`: series stacked over a shared x axis, from zero, as shares of 100%,
//! or as a streamgraph. Geometry comes from `hydroxy_charts::stacked_area`.

use dioxus::prelude::*;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use hydroxy_charts::stacked_area;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::{Curve, Series};
pub use hydroxy_charts::stacked_area::StackOffset;

use crate::plot_kit::{Plot, formatter, pct};

fn share(f: f64) -> String {
    format!("{:.0}%", f * 100.0)
}

/// Series stacked on top of each other so the total and each part read together. Hover or
/// touch to read every series at that x; focus the plot and use Left/Right to step,
/// Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn StackedAreaChart(
    /// The series, bottom of the stack first, one value per label. Past six, the smallest
    /// fold into "Other". Missing, negative or NaN values count as zero.
    series: ReadSignal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Traffic by source". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 240)]
    height: u16,
    /// Zero (default), Expand (100%), Wiggle (streamgraph) or Silhouette (centred).
    #[props(default)]
    offset: StackOffset,
    /// Straight segments (default) or a monotone curve.
    #[props(default)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points (x is a point index). Values are read on
    /// the stacked axis; with Expand they are shares, 0..=1.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// A value range the axis must cover, such as the shared domain of small multiples.
    y_domain: Option<(f64, f64)>,
) -> Element {
    let fmt = formatter(format);
    let names = labels();
    let series = plot::fold_series(&series(), 6);
    let n = names.len();
    let notes = annotations();
    // The shared domain only widens the axis: it reaches the layout as an unlabelled, undrawn band.
    let mut widened = notes.clone();
    widened.extend(y_domain.map(|(lo, hi)| Annotation::y_band(lo, hi, "")));
    let layout = stacked_area::layout(&series, n, offset, curve, &widened);
    let mut active = use_signal(|| None::<usize>);
    // The series under the pointer, if any.
    let mut layer = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let hovered = layer();

    let tip = current.map(|i| {
        let total = layout.totals[i];
        let mut rows: Vec<TipRow> = series
            .iter()
            .enumerate()
            .rev()
            .map(|(s, x)| {
                let v = x.values.get(i).copied();
                let text = match (offset, v) {
                    (StackOffset::Expand, Some(v)) if v.is_finite() && total > 0.0 => {
                        format!("{} ({})", fmt(v), share(v.max(0.0) / total))
                    }
                    _ => plot::cell(v, fmt),
                };
                TipRow { active: hovered == Some(s), ..TipRow::new(Some(colour(s)), x.name.clone(), text) }
            })
            .collect();
        rows.push(TipRow::new(None, "Total", fmt(total)));
        Tip { x: layout.xs[i], y: layout.tops.last().map_or(0.0, |t| t[i]), title: names[i].clone(), rows }
    });
    let mut head = vec![String::new()];
    head.extend(series.iter().map(|s| s.name.clone()));
    head.push("Total".into());
    let rows: Vec<Vec<String>> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let mut r = vec![name.clone()];
            r.extend(series.iter().map(|s| plot::cell(s.values.get(i).copied(), fmt)));
            r.push(fmt(layout.totals[i]));
            r
        })
        .collect();
    let legend = if series.len() > 1 {
        series.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect()
    } else {
        Vec::new()
    };
    let ticks = if offset == StackOffset::Expand { layout.axis.ticks(share) } else { layout.axis.ticks(fmt) };
    let xs = layout.xs.clone();
    let tops = layout.tops.clone();

    rsx! {
        Plot {
            scope: "stacked-area-chart",
            label,
            hint: hydroxy_charts::nav::HINT_STEP_LR,
            height,
            variant: offset.attr(),
            ticks,
            labels: plot::point_ticks(&names, &layout.xs, 8),
            annotations: plot::place(&notes, false, |x| plot::index_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head,
            rows,
            on_key: move |key: String| {
                layer.set(None);
                list_key(&key, active(), n).map(|next| active.set(next)).is_some()
            },
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, _)| nearest_index(&xs, x));
                let over = p.zip(next).and_then(|((_, y), i)| tops.iter().position(|t| t[i] >= y));
                if active() != next {
                    active.set(next);
                }
                if layer() != over {
                    layer.set(over);
                }
            },
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    for (i , d) in layout.areas.iter().enumerate() {
                        path { key: "{i}", "data-part": "area", "data-active": (hovered == Some(i)).then_some(""), d: "{d}", fill: colour(i) }
                    }
                }
            },
            overlay: current.map(|i| {
                let x = layout.xs[i];
                rsx! {
                    div { "data-part": "crosshair", style: "--p: {pct(x)};" }
                    for (s , t) in layout.tops.iter().enumerate() {
                        span { key: "{s}", "data-part": "dot", style: "--c: {x:.4}; --v: {t[i]:.4}; background: {colour(s)};" }
                    }
                }
            }),
        }
    }
}
