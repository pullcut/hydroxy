//! `RangeAreaChart`: a band from each point's low to its high, such as daily low and high
//! temperatures, with an optional line through a middle value. Geometry comes from
//! `hydroxy_charts::interval::band`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::interval;
use hydroxy_charts::line::x_positions;
use hydroxy_charts::nav::HINT_STEP_LR;
use hydroxy_charts::plot::{self, Key, Tip, TipRow};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::Interval;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, colour, formatter, pct};
use crate::rank_kit::{index_handlers, index_labels};

/// The band's legend swatch: the series hue, faded as the band is.
fn band_colour() -> String {
    format!("color-mix(in oklab, {} 35%, transparent)", colour(0))
}

/// A min–max band per x position, revealed left to right, with an optional middle line
/// (a mean or median). Hover or touch to read the low, high and spread; focus the plot and
/// use Left/Right to step, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RangeAreaChart(
    /// One range per x position: `Interval::new(label, mid, low, high)`. A NaN mid draws no
    /// line there; a NaN low or high leaves a gap in the band.
    data: ReadSignal<Vec<Interval>>,
    /// What the chart shows, such as "Daily temperature range". Used as the accessible name.
    label: String,
    /// Names the band in the legend, tooltip and table.
    #[props(default = "Range".to_string())]
    range_label: String,
    /// Names the middle line in the legend, tooltip and table.
    #[props(default = "Mean".to_string())]
    mid_label: String,
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
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = formatter(format);
    let items = data();
    let n = items.len();
    let notes = annotations();
    let pick = |f: fn(&Interval) -> f64| items.iter().map(f).collect::<Vec<f64>>();
    let layout = interval::band(&pick(|d| d.value), &pick(|d| d.low), &pick(|d| d.high), scale, curve, &notes);
    let has_mid = items.iter().any(|d| d.value.is_finite());
    let active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let xs = use_memo(move || x_positions(data.read().len(), None));
    let count = use_memo(move || data.read().len());
    let (on_point, on_key) = index_handlers(active, xs.into(), count.into());
    let at = |v: f64| v.is_finite().then(|| layout.axis.at(v));

    let now = current.map(|i| {
        let d = &items[i];
        EventDatum::category(i, d.label.clone(), if d.value.is_finite() { d.value } else { d.high })
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    let tip = current.map(|i| {
        let d = &items[i];
        let mut rows = vec![
            TipRow::new(Some(band_colour()), "High", plot::cell(Some(d.high), &fmt)),
            TipRow::new(Some(band_colour()), "Low", plot::cell(Some(d.low), &fmt)),
        ];
        if d.value.is_finite() {
            rows.push(TipRow::new(Some(colour(0)), mid_label.clone(), plot::cell(Some(d.value), &fmt)));
        }
        rows.push(TipRow {
            active: true,
            ..TipRow::new(None, range_label.clone(), plot::cell(Some(d.high - d.low), &fmt))
        });
        Tip { x: layout.xs[i], y: at(d.high).or(at(d.value)).unwrap_or(0.0), title: d.label.clone(), rows }
    });
    let names: Vec<String> = items.iter().map(|d| d.label.clone()).collect();
    let legend = if has_mid {
        vec![Key::new(range_label.clone(), band_colour()), Key::new(mid_label.clone(), colour(0))]
    } else {
        Vec::new()
    };
    let mut head = vec![String::new(), "Low".to_string(), "High".to_string()];
    if has_mid {
        head.push(mid_label.clone());
    }
    head.push(range_label.clone());
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|d| {
            let mut r = vec![d.label.clone(), plot::cell(Some(d.low), &fmt), plot::cell(Some(d.high), &fmt)];
            if has_mid {
                r.push(plot::cell(Some(d.value), &fmt));
            }
            r.push(plot::cell(Some(d.high - d.low), &fmt));
            r
        })
        .collect();

    rsx! {
        Plot {
            scope: "range-area-chart",
            label,
            hint: HINT_STEP_LR,
            height,
            ticks: layout.axis.ticks(&fmt),
            labels: index_labels(&names, &layout.xs),
            annotations: plot::place(&notes, false, |x| plot::index_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            datum: Some(datum),
            on_hover,
            on_select,
            head,
            rows,
            on_key,
            on_point,
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    path { "data-part": "band", d: "{layout.band}", fill: colour(0) }
                    if has_mid {
                        path { "data-part": "line", d: "{layout.line}", stroke: colour(0) }
                    }
                }
            },
            overlay: current.map(|i| {
                let d = &items[i];
                let x = layout.xs[i];
                rsx! {
                    div { "data-part": "crosshair", style: "--p: {pct(x)};" }
                    if let Some((lo , hi)) = at(d.low).zip(at(d.high)) {
                        div { "data-part": "span", style: "--c: {x:.4}; --a: {lo:.4}; --b: {hi:.4};" }
                    }
                    for (kind , v) in [("high", d.high), ("low", d.low), ("mid", d.value)] {
                        if let Some(y) = at(v) {
                            span { "data-part": "dot", "data-kind": kind, style: "--c: {x:.4}; --v: {y:.4};" }
                        }
                    }
                }
            }),
        }
    }
}
