//! `<ForecastFanChart>`: actual values, then a dashed forecast inside nested bands that
//! widen with the horizon. Geometry comes from `hydroxy_charts::interval::fan`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::interval;
use hydroxy_charts::nav::HINT_STEP_LR;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use leptos::prelude::*;

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::FanBand;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct, placed};
use crate::rank_kit::{index_handlers, index_labels};

/// How strongly band `i` of the fan shows, as a legend colour (bands overlap, so inner ones read darker).
fn band_colour(i: usize) -> String {
    format!("color-mix(in oklab, {} {}%, transparent)", colour(1), 16 * (i + 1))
}

/// History as a solid line, the forecast as a dashed one, and its uncertainty as nested
/// bands fanning out from the last actual value. Hover or touch to read a step; focus the
/// plot and use Left/Right to step, Home/End to jump, Escape to clear.
#[component]
pub fn ForecastFanChart(
    /// Actual values in x order. NaN leaves a gap.
    #[prop(into)]
    history: Signal<Vec<Datum>>,
    /// Forecast values for the steps after the history.
    #[prop(into)]
    forecast: Signal<Vec<Datum>>,
    /// Intervals around the forecast, one low and high per forecast step, widest first.
    #[prop(optional, into)]
    bands: Signal<Vec<FanBand>>,
    /// What the chart shows, such as "Weekly orders". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 240)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[prop(optional)]
    curve: Curve,
    /// Linear (default), Log or Symlog value axis.
    #[prop(optional)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a step index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Called when the pointer or keyboard moves to a step, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a step.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let steps =
        Memo::new(move |_| history.with(|h| forecast.with(|f| h.iter().chain(f).cloned().collect::<Vec<Datum>>())));
    let count = move || steps.with(Vec::len);
    let layout = Memo::new(move |_| {
        let values = |d: &Vec<Datum>| d.iter().map(|d| d.value).collect::<Vec<f64>>();
        let (h, f) = (history.with(values), forecast.with(values));
        bands.with(|b| annotations.with(|a| interval::fan(&h, &f, b, scale, curve, a)))
    });
    let active = RwSignal::new(None::<usize>);
    let range = move |b: &FanBand, j: usize| {
        format!("{} – {}", plot::cell(b.low.get(j).copied(), &fmt), plot::cell(b.high.get(j).copied(), &fmt))
    };
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let split = layout.with(|l| l.split);
        let d = steps.with(|s| s.get(i).cloned())?;
        let (series, index) = if i >= split { (1, i - split) } else { (0, i) };
        Some(EventDatum { series, index, label: Some(d.label), value: d.value, node: None })
    });

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = steps.with(|s| s.get(i).cloned())?;
        let (x, y, split) = layout.with(|l| (l.xs[i], l.points[i].unwrap_or(0.0), l.split));
        let mut rows = Vec::new();
        if i < split {
            rows.push(TipRow {
                active: true,
                ..TipRow::new(Some(colour(0)), "Actual", plot::cell(Some(d.value), &fmt))
            });
        } else {
            let step = i - split;
            rows.push(TipRow {
                active: true,
                ..TipRow::new(Some(colour(1)), "Forecast", plot::cell(Some(d.value), &fmt))
            });
            bands.with(|b| {
                rows.extend(
                    b.iter()
                        .enumerate()
                        .map(|(k, b)| TipRow::new(Some(band_colour(k)), b.name.clone(), range(b, step))),
                );
            });
        }
        Some(Tip { x, y, title: d.label, rows })
    });
    let xs = Signal::derive(move || layout.with(|l| l.xs.clone()));
    let (on_point, on_key) = index_handlers(active, xs, Signal::derive(count));
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(&fmt)));
    let names = Signal::derive(move || steps.with(|d| d.iter().map(|d| d.label.clone()).collect::<Vec<_>>()));
    let labels = index_labels(names, xs);
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, false, |x| plot::index_at(x, count()), |v| l.axis.scale.map(v))),
        annotations,
    );
    let legend = Signal::derive(move || {
        let mut keys = vec![Key::new("Actual", colour(0)), Key { dashed: true, ..Key::new("Forecast", colour(1)) }];
        bands.with(|b| keys.extend(b.iter().enumerate().map(|(k, b)| Key::new(b.name.clone(), band_colour(k)))));
        keys
    });
    let head = Signal::derive(move || {
        let mut h = vec![String::new(), "Actual".to_string(), "Forecast".to_string()];
        bands.with(|b| h.extend(b.iter().map(|b| b.name.clone())));
        h
    });
    let rows = Signal::derive(move || {
        let split = history.with(Vec::len);
        steps.with(|s| {
            bands.with(|b| {
                s.iter()
                    .enumerate()
                    .map(|(i, d)| {
                        let v = plot::cell(Some(d.value), &fmt);
                        let mut r = if i < split {
                            vec![d.label.clone(), v, "–".into()]
                        } else {
                            vec![d.label.clone(), "–".into(), v]
                        };
                        r.extend(b.iter().map(|b| if i < split { "–".into() } else { range(b, i - split) }));
                        r
                    })
                    .collect()
            })
        })
    });
    let marks = move || {
        let l = layout.get();
        let fans =
            l.bands.into_iter().map(|d| view! { <path data-part="band" d=d fill=colour(1)></path> }).collect_view();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {fans}
                <path data-part="line" d=l.actual stroke=colour(0)></path>
                <path data-part="line" data-dashed="" d=l.forecast stroke=colour(1)></path>
            </svg>
        }
    };
    let overlay = move || {
        active.get().and_then(|i| {
            let (x, y, split) = layout.with(|l| Some((l.xs[i], l.points[i]?, l.split)))?;
            Some(view! {
                <div data-part="crosshair" style=format!("--p: {};", pct(x))></div>
                <span data-part="dot" style=format!("--c: {x:.4}; --v: {y:.4}; background: {};", colour(usize::from(i >= split)))></span>
            })
        })
    };

    view! {
        <Plot
            scope="forecast-fan-chart"
            label
            hint=HINT_STEP_LR
            height
            ticks
            labels
            annotations=notes
            legend
            tip
            head
            rows
            on_key
            on_point
            marks
            datum=Some(datum)
            on_hover
            on_select
            overlay
        />
    }
}
