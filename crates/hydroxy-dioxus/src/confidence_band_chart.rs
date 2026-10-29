//! `ConfidenceBandChart`: an estimate as a line, with a shaded band from its low to its
//! high bound. Geometry comes from `hydroxy_charts::interval::band`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::interval;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::{self, list_key};
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::Interval;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct};

/// A line with its uncertainty: the band shows where the true value probably lies. Hover or
/// touch to read a point and its bounds; focus the plot and use Left/Right to step,
/// Home/End to jump, Escape to clear.
#[component]
pub fn ConfidenceBandChart(
    /// One estimate per x position, `Interval::new(label, value, low, high)`. NaN leaves a gap.
    data: ReadSignal<Vec<Interval>>,
    /// What the chart shows, such as "Daily conversion rate". Used as the accessible name.
    label: String,
    /// Names the line in the legend, tooltip and table.
    #[props(default = "Estimate".to_string())]
    value_label: String,
    /// Names the band in the legend, tooltip and table.
    #[props(default = "95% interval".to_string())]
    band_label: String,
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
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = formatter(format);
    let items = data();
    let n = items.len();
    let notes = annotations();
    let pick = |f: fn(&Interval) -> f64| items.iter().map(f).collect::<Vec<f64>>();
    let layout = interval::band(&pick(|d| d.value), &pick(|d| d.low), &pick(|d| d.high), scale, curve, &notes);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|i| EventDatum::category(i, items[i].label.clone(), items[i].value))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    let range = |d: &Interval| format!("{} – {}", plot::cell(Some(d.low), &fmt), plot::cell(Some(d.high), &fmt));

    let tip = current.map(|i| {
        let d = &items[i];
        let rows = vec![
            TipRow {
                active: true,
                ..TipRow::new(Some(colour(0)), value_label.clone(), plot::cell(Some(d.value), &fmt))
            },
            TipRow::new(Some(colour(0)), band_label.clone(), range(d)),
        ];
        Tip { x: layout.xs[i], y: layout.points[i].unwrap_or(0.0), title: d.label.clone(), rows }
    });
    let names: Vec<String> = items.iter().map(|d| d.label.clone()).collect();
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|d| {
            vec![
                d.label.clone(),
                plot::cell(Some(d.value), &fmt),
                plot::cell(Some(d.low), &fmt),
                plot::cell(Some(d.high), &fmt),
            ]
        })
        .collect();
    let legend = vec![
        Key::new(value_label.clone(), colour(0)),
        Key::new(band_label, format!("color-mix(in oklab, {} 30%, transparent)", colour(0))),
    ];
    let xs = layout.xs.clone();

    rsx! {
        Plot {
            scope: "confidence-band-chart",
            label,
            hint: nav::HINT_STEP_LR,
            height,
            ticks: layout.axis.ticks(&fmt),
            labels: plot::point_ticks(&names, &layout.xs, 8),
            annotations: plot::place(&notes, false, |x| plot::index_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            datum: Some(datum),
            on_hover,
            on_select,
            head: vec![String::new(), value_label, "Low".to_string(), "High".to_string()],
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
                    path { "data-part": "band", d: "{layout.band}", fill: colour(0) }
                    path { "data-part": "line", d: "{layout.line}", stroke: colour(0) }
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
