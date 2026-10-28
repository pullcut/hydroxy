//! `<LollipopChart>`: one value per category as a stick from the baseline and a dot.
//! Geometry comes from `hydroxy_charts::compare`.

use leptos::prelude::*;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tip, TipRow, colour};
use hydroxy_charts::compare;

pub use hydroxy_charts::{Datum, Orientation};
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, placed};

/// A lighter bar chart: a thin stick from zero and a dot at the value. Hover or touch a
/// category to read it; focus the plot and use the arrow keys to step through, Home/End
/// to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn LollipopChart(
    /// The values, in display order. NaN leaves a gap.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Response time by region". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels.
    #[prop(default = 220)]
    height: u16,
    /// Vertical (default) or horizontal sticks. Horizontal suits long category names.
    #[prop(optional)]
    orientation: Orientation,
    /// Linear (default), Log or Symlog value axis. On a log axis sticks start at the bottom.
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
    let horizontal = orientation == Orientation::Horizontal;
    let count = move || data.with(Vec::len);
    let layout = Memo::new(move |_| {
        let values: Vec<f64> = data.with(|d| d.iter().map(|d| d.value).collect());
        annotations.with(|a| compare::lollipops(&values, scale, a))
    });
    let active = RwSignal::new(None::<usize>);
    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let along = plot::band_centre(i, count());
        let value = layout.with(|l| l.heads.get(i).copied().flatten().unwrap_or(l.base));
        let rows = vec![TipRow::new(Some(colour(0)), "", plot::cell(Some(d.value), fmt))];
        Some(if horizontal { Tip { x: value, y: 1.0 - along, title: d.label, rows } } else { Tip { x: along, y: value, title: d.label, rows } })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, y)| plot::band_at(if horizontal { 1.0 - y } else { x }, count()));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        let key = if horizontal { plot::turn_key(&key) } else { &key };
        list_key(key, active.get_untracked(), count()).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || {
        let names: Vec<String> = data.with(|d| d.iter().map(|d| d.label.clone()).collect());
        plot::category_ticks(&names, if horizontal { usize::MAX } else { 10 })
    });
    let notes = placed(move |a| layout.with(|l| plot::place(a, horizontal, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v))), annotations);
    let head = Signal::derive(|| vec![String::new(), "Value".to_string()]);
    let rows = Signal::derive(move || data.with(|d| d.iter().map(|d| vec![d.label.clone(), plot::cell(Some(d.value), fmt)]).collect()));
    let indices = move || (0..count()).collect::<Vec<usize>>();
    let marks = move || {
        view! {
            <div data-part="bars">
                <For
                    each=indices
                    key=|i| *i
                    children=move |i| {
                        let head = Memo::new(move |_| layout.with(|l| (l.heads.get(i).copied().flatten(), l.base)));
                        let c = move || plot::band_centre(i, count());
                        view! {
                            <Show when=move || head.get().0.is_some()>
                                <span
                                    data-part="stick"
                                    style=move || {
                                        let (h, base) = head.get();
                                        let h = h.unwrap_or(base);
                                        format!("--c: {:.4}; --w: 0; --a: {:.4}; --b: {:.4}; --i: {i};", c(), base.min(h), base.max(h))
                                    }
                                ></span>
                                <span
                                    data-part="dot"
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    style=move || {
                                        format!("--c: {:.4}; --v: {:.4}; --i: {i}; background: {};", c(), head.get().0.unwrap_or(0.0), colour(0))
                                    }
                                ></span>
                            </Show>
                        }
                    }
                />
            </div>
        }
    };

    view! {
        <Plot
            scope="lollipop-chart"
            label
            hint="Use the arrow keys to move between values."
            height
            horizontal
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
