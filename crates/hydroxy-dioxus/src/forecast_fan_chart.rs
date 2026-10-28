//! `ForecastFanChart`: actual values, then a dashed forecast inside nested bands that
//! widen with the horizon. Geometry comes from `hydroxy_charts::interval::fan`.

use dioxus::prelude::*;
use hydroxy_charts::interval;
use hydroxy_charts::line::x_positions;
use hydroxy_charts::nav::HINT_STEP_LR;
use hydroxy_charts::plot::{self, Key, Tip, TipRow};

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::FanBand;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, colour, formatter, pct};
use crate::rank_kit::{index_handlers, index_labels};

/// How strongly band `i` of the fan shows, as a legend colour (bands overlap, so inner ones read darker).
fn band_colour(i: usize) -> String {
    format!("color-mix(in oklab, {} {}%, transparent)", colour(1), 16 * (i + 1))
}

/// History as a solid line, the forecast as a dashed one, and its uncertainty as nested
/// bands fanning out from the last actual value. Hover or touch to read a step; focus the
/// plot and use Left/Right to step, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ForecastFanChart(
    /// Actual values in x order. NaN leaves a gap.
    history: ReadSignal<Vec<Datum>>,
    /// Forecast values for the steps after the history.
    forecast: ReadSignal<Vec<Datum>>,
    /// Intervals around the forecast, one low and high per forecast step, widest first.
    #[props(default)]
    bands: ReadSignal<Vec<FanBand>>,
    /// What the chart shows, such as "Weekly orders". Used as the accessible name.
    label: String,
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
    /// Reference lines, bands and labelled points, in data units (x is a step index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let (past, future, fans) = (history(), forecast(), bands());
    let steps: Vec<Datum> = past.iter().chain(&future).cloned().collect();
    let n = steps.len();
    let notes = annotations();
    let values = |d: &[Datum]| d.iter().map(|d| d.value).collect::<Vec<f64>>();
    let layout = interval::fan(&values(&past), &values(&future), &fans, scale, curve, &notes);
    let split = layout.split;
    let active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let xs = use_memo(move || x_positions(history.read().len() + forecast.read().len(), None));
    let count = use_memo(move || history.read().len() + forecast.read().len());
    let (on_point, on_key) = index_handlers(active, xs.into(), count.into());
    let range = |b: &FanBand, j: usize| {
        format!("{} – {}", plot::cell(b.low.get(j).copied(), fmt), plot::cell(b.high.get(j).copied(), fmt))
    };

    let tip = current.map(|i| {
        let value = plot::cell(Some(steps[i].value), fmt);
        let mut rows = Vec::new();
        if i < split {
            rows.push(TipRow { active: true, ..TipRow::new(Some(colour(0)), "Actual", value) });
        } else {
            rows.push(TipRow { active: true, ..TipRow::new(Some(colour(1)), "Forecast", value) });
            rows.extend(
                fans.iter()
                    .enumerate()
                    .map(|(k, b)| TipRow::new(Some(band_colour(k)), b.name.clone(), range(b, i - split))),
            );
        }
        Tip { x: layout.xs[i], y: layout.points[i].unwrap_or(0.0), title: steps[i].label.clone(), rows }
    });
    let names: Vec<String> = steps.iter().map(|d| d.label.clone()).collect();
    let mut legend = vec![Key::new("Actual", colour(0)), Key { dashed: true, ..Key::new("Forecast", colour(1)) }];
    legend.extend(fans.iter().enumerate().map(|(k, b)| Key::new(b.name.clone(), band_colour(k))));
    let mut head = vec![String::new(), "Actual".to_string(), "Forecast".to_string()];
    head.extend(fans.iter().map(|b| b.name.clone()));
    let rows: Vec<Vec<String>> = steps
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let v = plot::cell(Some(d.value), fmt);
            let mut r =
                if i < split { vec![d.label.clone(), v, "–".into()] } else { vec![d.label.clone(), "–".into(), v] };
            r.extend(fans.iter().map(|b| if i < split { "–".into() } else { range(b, i - split) }));
            r
        })
        .collect();

    rsx! {
        Plot {
            scope: "forecast-fan-chart",
            label,
            hint: HINT_STEP_LR,
            height,
            ticks: layout.axis.ticks(fmt),
            labels: index_labels(&names, &layout.xs),
            annotations: plot::place(&notes, false, |x| plot::index_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head,
            rows,
            on_key,
            on_point,
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    for (k , d) in layout.bands.iter().enumerate() {
                        path { key: "{k}", "data-part": "band", d: "{d}", fill: colour(1) }
                    }
                    path { "data-part": "line", d: "{layout.actual}", stroke: colour(0) }
                    path { "data-part": "line", "data-dashed": "", d: "{layout.forecast}", stroke: colour(1) }
                }
            },
            overlay: current.and_then(|i| {
                let (x, y) = (layout.xs[i], layout.points[i]?);
                Some(rsx! {
                    div { "data-part": "crosshair", style: "--p: {pct(x)};" }
                    span { "data-part": "dot", style: "--c: {x:.4}; --v: {y:.4}; background: {colour(usize::from(i >= split))};" }
                })
            }),
        }
    }
}
