//! `<SplitLineChart>`: one line, solid for actual values and dashed for the projection
//! after a split index. Geometry comes from `hydroxy_charts::split::split_line`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use hydroxy_charts::split;
use leptos::prelude::*;

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct, placed};

/// Actual values as a solid line, then a projection as a dashed one, joined at the last
/// actual point. Hover or touch to read a point; focus the plot and use Left/Right to step,
/// Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SplitLineChart(
    /// The values in x order, actual first. NaN leaves a gap.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// Index of the first projected value. Past the end, the whole line is actual.
    #[prop(into)]
    split: Signal<usize>,
    /// What the chart shows, such as "Monthly revenue". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names the solid part in the legend, tooltip and table.
    #[prop(default = "Actual".into(), into)]
    actual_label: String,
    /// Names the dashed part in the legend, tooltip and table.
    #[prop(default = "Projected".into(), into)]
    projected_label: String,
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
    /// Reference lines, bands and labelled points, in data units (x is a point index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Called when the hovered or focused point changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a point.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || data.with(Vec::len);
    let layout = Memo::new(move |_| {
        let values: Vec<f64> = data.with(|d| d.iter().map(|d| d.value).collect());
        annotations.with(|a| split::split_line(&values, split.get(), scale, curve, a))
    });
    let active = RwSignal::new(None::<usize>);
    let names = StoredValue::new((actual_label, projected_label));
    let kind = move |i: usize| {
        let (actual, projected) = names.get_value();
        if i < split.get() { actual } else { projected }
    };

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let (x, y) = layout.with(|l| (l.xs[i], l.points[i].unwrap_or(0.0)));
        let rows =
            vec![TipRow { active: true, ..TipRow::new(Some(colour(0)), kind(i), plot::cell(Some(d.value), &fmt)) }];
        Some(Tip { x, y, title: d.label, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| layout.with(|l| nearest_index(&l.xs, x)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some()
    });
    let datum = Signal::derive(move || {
        let i = active.get()?;
        data.with(|d| d.get(i).map(|d| EventDatum::category(i, d.label.clone(), d.value)))
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(&fmt)));
    let labels = Signal::derive(move || {
        data.with(|d| {
            layout.with(|l| plot::point_ticks(&d.iter().map(|d| d.label.clone()).collect::<Vec<_>>(), &l.xs, 8))
        })
    });
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, false, |x| plot::index_at(x, count()), |v| l.axis.scale.map(v))),
        annotations,
    );
    let legend = Signal::derive(move || {
        let (actual, projected) = names.get_value();
        vec![Key::new(actual, colour(0)), Key { dashed: true, ..Key::new(projected, colour(0)) }]
    });
    let head = Signal::derive(|| vec![String::new(), "Value".to_string(), "Kind".to_string()]);
    let rows = Signal::derive(move || {
        data.with(|d| {
            d.iter().enumerate().map(|(i, d)| vec![d.label.clone(), plot::cell(Some(d.value), &fmt), kind(i)]).collect()
        })
    });
    let marks = move || {
        let l = layout.get();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                <path data-part="line" d=l.before stroke=colour(0)></path>
                <path data-part="line" data-dashed="" d=l.after stroke=colour(0)></path>
            </svg>
        }
    };
    let overlay = move || {
        active.get().and_then(|i| {
            let (x, y) = layout.with(|l| Some((l.xs[i], l.points[i]?)))?;
            Some(view! {
                <div data-part="crosshair" style=format!("--p: {};", pct(x))></div>
                <span data-part="dot" style=format!("--c: {x:.4}; --v: {y:.4}; background: {};", colour(0))></span>
            })
        })
    };

    view! {
        <Plot
            scope="split-line-chart"
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
            overlay
            datum=Some(datum)
            on_hover
            on_select
        />
    }
}
