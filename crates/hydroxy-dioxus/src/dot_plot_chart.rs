//! `DotPlotChart`: one row per category with a dot per series; two or more series draw
//! a dumbbell bar between them. Geometry comes from `hydroxy_charts::compare`.

use dioxus::prelude::*;
use hydroxy_charts::compare;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, colour, formatter};

/// Categories down the left, values along the bottom, a dot per series. With two or more
/// series a bar joins each row's lowest and highest dot (a dumbbell), so the gap reads at a
/// glance. Hover or touch a dot to read its row; focus the plot and use Up/Down to move
/// between rows, Left/Right between series, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn DotPlotChart(
    /// One label per row.
    rows: ReadSignal<Vec<String>>,
    /// The series, one value per row, in categorical colour order. Past six, the smallest
    /// fold into "Other". NaN leaves the dot out.
    series: ReadSignal<Vec<Series>>,
    /// What the chart shows, such as "Median salary by role, 2024 and 2025". Used as the accessible name.
    label: String,
    /// Plot height in px. Default: 36px per row.
    height: Option<u16>,
    /// Linear (default), Log or Symlog value axis. The axis fits the data rather than starting at zero.
    #[props(default)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is the value, y a row index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let names = rows();
    let series = plot::fold_series(&series(), 6);
    let (n, m) = (names.len(), series.len());
    let notes = annotations();
    let layout = compare::dots(&series, n, scale, &notes);
    let height = height.unwrap_or_else(|| u16::try_from(n * 36).unwrap_or(u16::MAX).max(108));
    // (row, series), matching nav::grid_key's (row, column).
    let mut active = use_signal(|| None::<(usize, usize)>);
    let current = active().filter(|&(r, s)| r < n && s < m);
    let tip = current.and_then(|(r, s)| {
        let x = layout.dots[r][s].or(layout.spans[r].map(|sp| sp.1))?;
        let rows = series
            .iter()
            .enumerate()
            .map(|(i, x)| TipRow {
                active: i == s,
                ..TipRow::new(Some(colour(i)), x.name.clone(), plot::cell(x.values.get(r).copied(), fmt))
            })
            .collect();
        Some(Tip { x, y: 1.0 - plot::band_centre(r, n), title: names[r].clone(), rows })
    });
    let legend: Vec<Key> = if m > 1 {
        series.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect()
    } else {
        Vec::new()
    };
    let head: Vec<String> = std::iter::once(String::new()).chain(series.iter().map(|s| s.name.clone())).collect();
    let table: Vec<Vec<String>> = names
        .iter()
        .enumerate()
        .map(|(r, name)| {
            std::iter::once(name.clone())
                .chain(series.iter().map(|x| plot::cell(x.values.get(r).copied(), fmt)))
                .collect()
        })
        .collect();
    let dots_for_pointer = layout.dots.clone();

    rsx! {
        Plot {
            scope: "dot-plot-chart",
            label,
            hint: "Use the up and down arrow keys to move between rows, left and right between series.",
            height,
            horizontal: true,
            ticks: layout.axis.ticks(fmt),
            labels: plot::category_ticks(&names, usize::MAX),
            annotations: plot::place(&notes, true, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head,
            rows: table,
            on_key: move |key: String| grid_key(&key, active(), n, m).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| {
                    let r = plot::band_at(1.0 - y, n)?;
                    compare::nearest_dot(&dots_for_pointer[r], x).map(|s| (r, s))
                });
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "bars",
                    for (r , (row , span)) in layout.dots.iter().zip(&layout.spans).enumerate() {
                        if let Some((lo , hi)) = *span {
                            span {
                                "data-part": "stick",
                                style: "--c: {plot::band_centre(r, n):.4}; --w: 0; --a: {lo:.4}; --b: {hi:.4}; --i: {r};",
                            }
                        }
                        for (s , v) in row.iter().enumerate() {
                            if let Some(v) = *v {
                                span {
                                    "data-part": "dot",
                                    "data-index": "{s}",
                                    "data-active": (current == Some((r, s))).then_some(""),
                                    style: "--c: {plot::band_centre(r, n):.4}; --v: {v:.4}; --i: {r}; background: {colour(s)};",
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}
