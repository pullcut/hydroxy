//! `<ErrorBarChart>`: an estimate per category with low and high whiskers.
//! Geometry comes from `hydroxy_charts::interval`.

use hydroxy_charts::interval;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tip, TipRow, colour};
use leptos::prelude::*;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::Interval;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, placed};

/// Share of each category band a bar fills.
const BAR: f64 = 0.5;

/// Estimates with their uncertainty: a dot (or a bar from zero) per category and a capped
/// whisker from low to high. Hover or touch a category to read all three; focus the plot
/// and use Left/Right to step through, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ErrorBarChart(
    /// The estimates, in display order: `Interval::new(label, value, low, high)`.
    #[prop(into)]
    data: Signal<Vec<Interval>>,
    /// What the chart shows, such as "Conversion rate by variant". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels.
    #[prop(default = 240)]
    height: u16,
    /// Draw a bar from zero to each estimate instead of a dot.
    #[prop(optional)]
    bars: bool,
    /// Linear (default), Log or Symlog value axis.
    #[prop(optional)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a category index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || data.with(Vec::len);
    let layout = Memo::new(move |_| data.with(|d| annotations.with(|a| interval::whiskers(d, scale, bars, a))));
    let active = RwSignal::new(None::<usize>);
    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let w = layout.with(|l| l.whiskers.get(i).copied())?;
        let rows = vec![
            TipRow::new(Some(colour(0)), "Estimate", plot::cell(Some(d.value), fmt)),
            TipRow::new(None, "Low", plot::cell(Some(d.low), fmt)),
            TipRow::new(None, "High", plot::cell(Some(d.high), fmt)),
        ];
        let y = [w.value, w.low, w.high].into_iter().flatten().fold(0.0, f64::max);
        Some(Tip { x: plot::band_centre(i, count()), y, title: d.label, rows })
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
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || {
        plot::category_ticks(&data.with(|d| d.iter().map(|d| d.label.clone()).collect::<Vec<_>>()), 12)
    });
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, false, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v))),
        annotations,
    );
    let head = Signal::derive(|| ["", "Estimate", "Low", "High"].map(String::from).to_vec());
    let rows = Signal::derive(move || {
        data.with(|d| {
            d.iter()
                .map(|d| {
                    vec![
                        d.label.clone(),
                        plot::cell(Some(d.value), fmt),
                        plot::cell(Some(d.low), fmt),
                        plot::cell(Some(d.high), fmt),
                    ]
                })
                .collect()
        })
    });
    let marks = move || {
        let l = layout.get();
        let n = l.whiskers.len();
        #[allow(clippy::cast_precision_loss, reason = "category counts are small")]
        let width = BAR / n.max(1) as f64;
        let base = l.axis.base();
        l.whiskers
            .into_iter()
            .enumerate()
            .map(|(i, w)| {
                let c = plot::band_centre(i, n);
                let is_active = move || (active.get() == Some(i)).then_some("");
                let bar = w.value.filter(|_| bars).map(|v| {
                    let style = format!("--c: {c:.4}; --w: {width:.4}; --a: {:.4}; --b: {:.4}; --i: {i};", base.min(v), base.max(v));
                    view! { <div data-part="bar" data-negative=(v < base).then_some("") data-active=is_active style=style></div> }
                });
                let whisker = w.low.zip(w.high).map(|(lo, hi)| {
                    let style = format!("--c: {c:.4}; --w: 0; --a: {:.4}; --b: {:.4}; --i: {i};", lo.min(hi), lo.max(hi));
                    view! { <span data-part="whisker" style=style></span> }
                });
                let dot = w.value.filter(|_| !bars).map(|v| {
                    let style = format!("--c: {c:.4}; --v: {v:.4}; --i: {i};");
                    view! { <span data-part="dot" data-active=is_active style=style></span> }
                });
                view! { {bar}{whisker}{dot} }
            })
            .collect_view()
    };
    let marks = move || view! { <div data-part="bars">{marks}</div> };

    view! {
        <Plot
            scope="error-bar-chart"
            label
            hint="Use the left and right arrow keys to step through."
            height
            variant=if bars { "bars" } else { "points" }
            ticks
            labels
            annotations=notes
            tip
            head
            rows
            on_key
            on_point
            marks
        />
    }
}
