//! `GroupedBarChart`: several series side by side in each category. Geometry comes from
//! `oxide_charts::grouped`; bars are positioned with CSS percentages.

use dioxus::prelude::*;
use oxide_charts::grouped;
use oxide_charts::nav::grid_key;
use oxide_charts::plot::{self, Key, Tip, TipRow, colour};

pub use oxide_charts::Orientation;
pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::line::Series;

use crate::plot_kit::{Plot, formatter};

/// Series side by side within each category, from a zero baseline. Hover or touch a bar to
/// read its whole group; focus the plot and use the arrow keys to move between bars
/// (along the categories, and across series), Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn GroupedBarChart(
    /// One label per category.
    categories: ReadSignal<Vec<String>>,
    /// The series, one value per category, in categorical colour order. Past six, the
    /// smallest fold into "Other". NaN leaves an empty slot.
    series: ReadSignal<Vec<Series>>,
    /// What the chart shows, such as "Signups by channel". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 240)]
    height: u16,
    /// Vertical (default) or horizontal bars. Horizontal suits long category names.
    #[props(default)]
    orientation: Orientation,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a category index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let horizontal = orientation == Orientation::Horizontal;
    let names = categories();
    let series = plot::fold_series(&series(), 6);
    let n = names.len();
    let m = series.len();
    let notes = annotations();
    let layout = grouped::layout(&series, n, &notes);
    // (series, category), matching nav::grid_key's (row, column).
    let mut active = use_signal(|| None::<(usize, usize)>);
    let current = active().filter(|&(s, c)| s < m && c < n);

    let tip = current.map(|(s, c)| {
        let bar = layout.bars.iter().find(|b| (b.series, b.category) == (s, c));
        let along = bar.map_or_else(|| plot::band_centre(c, n), |b| b.centre);
        let value = bar.map_or_else(|| layout.axis.base(), |b| b.to.max(b.from));
        let rows = series
            .iter()
            .enumerate()
            .map(|(i, x)| TipRow { active: i == s, ..TipRow::new(Some(colour(i)), x.name.clone(), plot::cell(x.values.get(c).copied(), fmt)) })
            .collect();
        let title = names[c].clone();
        if horizontal { Tip { x: value, y: 1.0 - along, title, rows } } else { Tip { x: along, y: value, title, rows } }
    });
    let legend: Vec<Key> = if m > 1 { series.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect() } else { Vec::new() };
    let head: Vec<String> = std::iter::once(String::new()).chain(series.iter().map(|s| s.name.clone())).collect();
    let rows: Vec<Vec<String>> =
        names.iter().enumerate().map(|(c, name)| std::iter::once(name.clone()).chain(series.iter().map(|x| plot::cell(x.values.get(c).copied(), fmt))).collect()).collect();

    rsx! {
        Plot {
            scope: "grouped-bar-chart",
            label,
            hint: "Use the arrow keys to move between bars.",
            height,
            horizontal,
            ticks: layout.axis.ticks(fmt),
            labels: plot::category_ticks(&names, if horizontal { usize::MAX } else { 8 }),
            annotations: plot::place(&notes, horizontal, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head,
            rows,
            on_key: move |key: String| {
                let key = if horizontal { plot::turn_key(&key) } else { &key };
                grid_key(key, active(), m, n).map(|next| active.set(next)).is_some()
            },
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| grouped::pick(if horizontal { 1.0 - y } else { x }, n, m));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "bars",
                    for b in layout.bars.iter() {
                        div {
                            key: "{b.series}-{b.category}",
                            "data-part": "bar",
                            "data-index": "{b.series}",
                            "data-negative": (b.to < b.from).then_some(""),
                            "data-active": (current == Some((b.series, b.category))).then_some(""),
                            style: "--c: {b.centre:.4}; --w: {b.width:.4}; --a: {b.from.min(b.to):.4}; --b: {b.from.max(b.to):.4}; --i: {b.category}; background: {colour(b.series)};",
                        }
                    }
                }
            },
        }
    }
}
