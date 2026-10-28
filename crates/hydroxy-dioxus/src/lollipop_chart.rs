//! `LollipopChart`: one value per category as a stick from the baseline and a dot.
//! Geometry comes from `hydroxy_charts::compare`.

use dioxus::prelude::*;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tip, TipRow, colour};
use hydroxy_charts::compare;

pub use hydroxy_charts::{Datum, Orientation};
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter};

/// A lighter bar chart: a thin stick from zero and a dot at the value. Hover or touch a
/// category to read it; focus the plot and use the arrow keys to step through, Home/End
/// to jump, Escape to clear.
#[component]
pub fn LollipopChart(
    /// The values, in display order. NaN leaves a gap.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Response time by region". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels.
    #[props(default = 220)]
    height: u16,
    /// Vertical (default) or horizontal sticks. Horizontal suits long category names.
    #[props(default)]
    orientation: Orientation,
    /// Linear (default), Log or Symlog value axis. On a log axis sticks start at the bottom.
    #[props(default)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a category index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let horizontal = orientation == Orientation::Horizontal;
    let items = data();
    let n = items.len();
    let notes = annotations();
    let values: Vec<f64> = items.iter().map(|d| d.value).collect();
    let layout = compare::lollipops(&values, scale, &notes);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let tip = current.map(|i| {
        let along = plot::band_centre(i, n);
        let value = layout.heads[i].unwrap_or(layout.base);
        let rows = vec![TipRow::new(Some(colour(0)), "", plot::cell(Some(items[i].value), fmt))];
        let title = items[i].label.clone();
        if horizontal { Tip { x: value, y: 1.0 - along, title, rows } } else { Tip { x: along, y: value, title, rows } }
    });
    let names: Vec<String> = items.iter().map(|d| d.label.clone()).collect();
    let rows: Vec<Vec<String>> = items.iter().map(|d| vec![d.label.clone(), plot::cell(Some(d.value), fmt)]).collect();
    let base = layout.base;

    rsx! {
        Plot {
            scope: "lollipop-chart",
            label,
            hint: "Use the arrow keys to move between values.",
            height,
            horizontal,
            ticks: layout.axis.ticks(fmt),
            labels: plot::category_ticks(&names, if horizontal { usize::MAX } else { 10 }),
            annotations: plot::place(&notes, horizontal, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            tip,
            head: vec![String::new(), "Value".to_string()],
            rows,
            on_key: move |key: String| {
                let key = if horizontal { plot::turn_key(&key) } else { &key };
                list_key(key, active(), n).map(|next| active.set(next)).is_some()
            },
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| plot::band_at(if horizontal { 1.0 - y } else { x }, n));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "bars",
                    for (i , h) in layout.heads.iter().enumerate() {
                        if let Some(h) = *h {
                            span {
                                key: "s{i}",
                                "data-part": "stick",
                                style: "--c: {plot::band_centre(i, n):.4}; --w: 0; --a: {base.min(h):.4}; --b: {base.max(h):.4}; --i: {i};",
                            }
                            span {
                                "data-part": "dot",
                                "data-active": (current == Some(i)).then_some(""),
                                style: "--c: {plot::band_centre(i, n):.4}; --v: {h:.4}; --i: {i}; background: {colour(0)};",
                            }
                        }
                    }
                }
            },
        }
    }
}
