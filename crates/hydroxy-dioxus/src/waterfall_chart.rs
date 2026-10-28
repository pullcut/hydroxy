//! `WaterfallChart`: how a starting value becomes an end value, step by step.
//! Geometry comes from `hydroxy_charts::waterfall`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow};
use hydroxy_charts::waterfall::{self, StepKind};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::waterfall::Step;

use crate::plot_kit::{Plot, formatter};
use crate::sparkline::use_datum;

/// Share of each category band a bar fills.
const BAR: f64 = 0.64;

/// Running totals as floating bars: rises in the success colour, falls in the danger
/// colour, and optional total bars from zero, joined by dashed connectors. Hover or touch a
/// step to read it; focus the plot and use Left/Right to step through, Home/End to jump,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn WaterfallChart(
    /// The steps in order: `Step::change(label, value)`, `Step::total(label)`, or `Step::start(label, value)` for an opening balance.
    steps: ReadSignal<Vec<Step>>,
    /// What the chart shows, such as "Revenue bridge". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a step index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Called when the hovered or focused step changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a step.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = formatter(format);
    let items = steps();
    let n = items.len();
    let notes = annotations();
    let layout = waterfall::waterfall(&items, &notes);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    // `value` is the step's change, or the running total for a total bar.
    let datum = use_datum(current.map(|i| {
        let bar = layout.bars[i];
        let value = if bar.kind == StepKind::Total { bar.end } else { bar.end - bar.start };
        EventDatum::category(i, items[i].label.clone(), value)
    }));
    let signed = move |v: f64| if v > 0.0 { format!("+{}", fmt(v)) } else { fmt(v) };
    let tip = current.map(|i| {
        let bar = layout.bars[i];
        let swatch = Some(bar.kind.colour().to_string());
        let rows = if bar.kind == StepKind::Total {
            vec![TipRow::new(swatch, "Total", fmt(bar.end))]
        } else {
            vec![
                TipRow::new(swatch, "Change", signed(bar.end - bar.start)),
                TipRow::new(None, "Running total", fmt(bar.end)),
            ]
        };
        Tip { x: plot::band_centre(i, n), y: bar.from.max(bar.to), title: items[i].label.clone(), rows }
    });
    let mut legend =
        vec![Key::new("Increase", StepKind::Increase.colour()), Key::new("Decrease", StepKind::Decrease.colour())];
    if items.iter().any(|s| s.total) {
        legend.push(Key::new("Total", StepKind::Total.colour()));
    }
    let rows: Vec<Vec<String>> = items
        .iter()
        .zip(&layout.bars)
        .map(|(s, b)| {
            let change = if b.kind == StepKind::Total { "–".to_string() } else { signed(b.end - b.start) };
            vec![s.label.clone(), change, fmt(b.end)]
        })
        .collect();
    let names: Vec<String> = items.iter().map(|s| s.label.clone()).collect();
    #[allow(clippy::cast_precision_loss, reason = "step counts are small")]
    let band = 1.0 / n.max(1) as f64;

    rsx! {
        Plot {
            scope: "waterfall-chart",
            label,
            hint: hydroxy_charts::nav::HINT_STEP_LR,
            height,
            ticks: layout.axis.ticks(fmt),
            labels: plot::category_ticks(&names, 12),
            annotations: plot::place(&notes, false, |x| plot::category_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            datum,
            on_hover,
            on_select,
            head: vec![String::new(), "Change".to_string(), "Running total".to_string()],
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, _)| plot::band_at(x, n));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "bars",
                    for (i , pair) in layout.bars.windows(2).enumerate() {
                        span {
                            key: "c{i}",
                            "data-part": "connector",
                            style: "--c: {plot::band_centre(i, n) + band * BAR / 2.0:.4}; --w: {band * (1.0 - BAR):.4}; --v: {pair[0].to:.4};",
                        }
                    }
                    for (i , b) in layout.bars.iter().enumerate() {
                        div {
                            key: "b{i}",
                            "data-part": "bar",
                            "data-kind": b.kind.attr(),
                            "data-negative": (b.to < b.from).then_some(""),
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--c: {plot::band_centre(i, n):.4}; --w: {band * BAR:.4}; --a: {b.from.min(b.to):.4}; --b: {b.from.max(b.to):.4}; --i: {i};",
                        }
                    }
                }
            },
        }
    }
}
