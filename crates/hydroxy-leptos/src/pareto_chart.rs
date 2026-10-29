//! `<ParetoChart>`: categories sorted largest first, with the running share of the total.
//! Geometry comes from `hydroxy_charts::waterfall::pareto`.
//!
//! This is the one cartesian chart with a second axis: the cumulative line is read
//! against a labelled 0–100% scale on the right, since a share and a count cannot share
//! one axis honestly.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::{HINT_STEP_LR, list_key};
use hydroxy_charts::plot::{self, Key, Tick, Tip, TipRow};
use hydroxy_charts::waterfall;
use leptos::prelude::*;

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::annotate::Annotation;

use crate::plot_kit::{Plot, colour, formatter, placed};

/// Share of each category band a bar fills.
const BAR: f64 = 0.64;

fn share(f: f64) -> String {
    format!("{:.0}%", f * 100.0)
}

/// Bars sorted largest first, plus a line of their cumulative share of the total, read on a
/// 0–100% axis on the right (the one accepted exception to the single-axis rule). Hover or
/// touch a category to read both; focus the plot and use Left/Right to step through,
/// Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn ParetoChart(
    /// The values, in any order; the chart sorts them. Negative or NaN values count as zero.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Support tickets by cause". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// What the bars count, for the legend, tooltip and table.
    #[prop(default = "Count".into(), into)]
    value_label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 240)]
    height: u16,
    /// Formats values for the left axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, on the left axis (x is a position in sorted order).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || data.with(Vec::len);
    let layout = Memo::new(move |_| {
        let values: Vec<f64> = data.with(|d| d.iter().map(|d| d.value).collect());
        annotations.with(|a| waterfall::pareto(&values, a))
    });
    let sorted = move || -> Vec<Datum> {
        layout.with(|l| data.with(|d| l.order.iter().filter_map(|&i| d.get(i).cloned()).collect()))
    };
    let active = RwSignal::new(None::<usize>);
    let name = StoredValue::new(value_label.clone());
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let at = layout.with(|l| l.order.get(i).copied())?;
        let d = data.with(|d| d.get(at).cloned())?;
        Some(EventDatum::category(at, d.label, d.value))
    });
    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = sorted().get(i).cloned()?;
        let (bar, cum) = layout.with(|l| (l.bars.get(i).copied(), l.cumulative.get(i).copied()));
        let (bar, cum) = (bar?, cum?);
        let rows = vec![
            TipRow::new(Some(colour(0)), name.get_value(), fmt(d.value)),
            TipRow::new(Some(colour(1)), "Cumulative", share(cum)),
        ];
        Some(Tip { x: plot::band_centre(i, count()), y: bar.max(cum), title: d.label, rows })
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
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(&fmt)));
    let right_ticks =
        Signal::derive(|| [0.0, 0.25, 0.5, 0.75, 1.0].map(|at| Tick { at, text: share(at), baseline: false }).to_vec());
    let labels =
        Signal::derive(move || plot::category_ticks(&sorted().into_iter().map(|d| d.label).collect::<Vec<_>>(), 12));
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, false, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v))),
        annotations,
    );
    let legend =
        Signal::derive(move || vec![Key::new(name.get_value(), colour(0)), Key::new("Cumulative share", colour(1))]);
    let head = Signal::derive(move || vec![String::new(), name.get_value(), "Cumulative %".to_string()]);
    let rows = Signal::derive(move || {
        let cumulative = layout.with(|l| l.cumulative.clone());
        sorted().into_iter().zip(cumulative).map(|(d, c)| vec![d.label, fmt(d.value), share(c)]).collect()
    });
    let marks = move || {
        let l = layout.get();
        let n = l.bars.len();
        #[allow(clippy::cast_precision_loss, reason = "category counts are small")]
        let width = BAR / n.max(1) as f64;
        let base = l.axis.base();
        let bars = l
            .bars
            .iter()
            .enumerate()
            .map(|(i, &h)| {
                let style = format!("--c: {:.4}; --w: {width:.4}; --a: {base:.4}; --b: {h:.4}; --i: {i}; background: {};", plot::band_centre(i, n), colour(0));
                view! { <div data-part="bar" data-active=move || (active.get() == Some(i)).then_some("") style=style></div> }
            })
            .collect_view();
        let dots = l
            .cumulative
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let style = format!("--c: {:.4}; --v: {c:.4}; --i: {i}; background: {};", plot::band_centre(i, n), colour(1));
                view! { <span data-part="dot" data-active=move || (active.get() == Some(i)).then_some("") style=style></span> }
            })
            .collect_view();
        view! {
            <div data-part="bars">{bars}</div>
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                <path data-part="line" d=l.line stroke=colour(1)></path>
            </svg>
            <div data-part="points">{dots}</div>
        }
    };

    view! {
        <Plot
            scope="pareto-chart"
            label
            hint=HINT_STEP_LR
            height
            ticks
            right_ticks
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
