//! `<WaterfallChart>`: how a starting value becomes an end value, step by step.
//! Geometry comes from `hydroxy_charts::waterfall`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow};
use hydroxy_charts::waterfall::{self, StepKind};
use leptos::prelude::*;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::waterfall::Step;

use crate::plot_kit::{Plot, formatter, placed};

/// Share of each category band a bar fills.
const BAR: f64 = 0.64;

/// Running totals as floating bars: rises in the success colour, falls in the danger
/// colour, and optional total bars from zero, joined by dashed connectors. Hover or touch a
/// step to read it; focus the plot and use Left/Right to step through, Home/End to jump,
/// Escape to clear.
#[component]
pub fn WaterfallChart(
    /// The steps in order: `Step::change(label, value)`, `Step::total(label)`, or `Step::start(label, value)` for an opening balance.
    #[prop(into)]
    steps: Signal<Vec<Step>>,
    /// What the chart shows, such as "Revenue bridge". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a step index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Called when the hovered or focused step changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a step.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || steps.with(Vec::len);
    let layout = Memo::new(move |_| steps.with(|s| annotations.with(|a| waterfall::waterfall(s, a))));
    let active = RwSignal::new(None::<usize>);
    let signed = move |v: f64| if v > 0.0 { format!("+{}", fmt(v)) } else { fmt(v) };
    let tip = Signal::derive(move || {
        let i = active.get()?;
        let step = steps.with(|s| s.get(i).cloned())?;
        let bar = layout.with(|l| l.bars.get(i).copied())?;
        let swatch = Some(bar.kind.colour().to_string());
        let rows = if bar.kind == StepKind::Total {
            vec![TipRow::new(swatch, "Total", fmt(bar.end))]
        } else {
            vec![
                TipRow::new(swatch, "Change", signed(bar.end - bar.start)),
                TipRow::new(None, "Running total", fmt(bar.end)),
            ]
        };
        Some(Tip { x: plot::band_centre(i, count()), y: bar.from.max(bar.to), title: step.label, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| plot::band_at(x, count()));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some()
    });
    // `value` is the step's change, or the running total for a total bar.
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let label = steps.with(|s| s.get(i).map(|s| s.label.clone()))?;
        let bar = layout.with(|l| l.bars.get(i).copied())?;
        let value = if bar.kind == StepKind::Total { bar.end } else { bar.end - bar.start };
        Some(EventDatum::category(i, label, value))
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(&fmt)));
    let labels = Signal::derive(move || {
        let names: Vec<String> = steps.with(|s| s.iter().map(|s| s.label.clone()).collect());
        plot::category_ticks(&names, 12)
    });
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, false, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v))),
        annotations,
    );
    let legend = Signal::derive(move || {
        let mut keys =
            vec![Key::new("Increase", StepKind::Increase.colour()), Key::new("Decrease", StepKind::Decrease.colour())];
        if steps.with(|s| s.iter().any(|s| s.total)) {
            keys.push(Key::new("Total", StepKind::Total.colour()));
        }
        keys
    });
    let head = Signal::derive(|| vec![String::new(), "Change".to_string(), "Running total".to_string()]);
    let rows = Signal::derive(move || {
        steps.with(|s| {
            layout.with(|l| {
                s.iter()
                    .zip(&l.bars)
                    .map(|(s, b)| {
                        let change =
                            if b.kind == StepKind::Total { "–".to_string() } else { signed(b.end - b.start) };
                        vec![s.label.clone(), change, fmt(b.end)]
                    })
                    .collect()
            })
        })
    });
    let marks = move || {
        let n = count();
        #[allow(clippy::cast_precision_loss, reason = "step counts are small")]
        let band = 1.0 / n.max(1) as f64;
        let bars = layout.get().bars;
        let connectors = bars
            .windows(2)
            .enumerate()
            .map(|(i, pair)| {
                let style = format!(
                    "--c: {:.4}; --w: {:.4}; --v: {:.4};",
                    plot::band_centre(i, n) + band * BAR / 2.0,
                    band * (1.0 - BAR),
                    pair[0].to
                );
                view! { <span data-part="connector" style=style></span> }
            })
            .collect_view();
        let bars = bars
            .into_iter()
            .enumerate()
            .map(|(i, b)| {
                let style = format!(
                    "--c: {:.4}; --w: {:.4}; --a: {:.4}; --b: {:.4}; --i: {i};",
                    plot::band_centre(i, n),
                    band * BAR,
                    b.from.min(b.to),
                    b.from.max(b.to),
                );
                view! {
                    <div
                        data-part="bar"
                        data-kind=b.kind.attr()
                        data-negative=(b.to < b.from).then_some("")
                        data-active=move || (active.get() == Some(i)).then_some("")
                        style=style
                    ></div>
                }
            })
            .collect_view();
        view! { <div data-part="bars">{connectors}{bars}</div> }
    };

    view! {
        <Plot
            scope="waterfall-chart"
            label
            hint=hydroxy_charts::nav::HINT_STEP_LR
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
        />
    }
}
