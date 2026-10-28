//! `<ConnectedScatterChart>`: two measures against each other, joined in time order.
//! Geometry comes from `hydroxy_charts::connected`.

use leptos::prelude::*;
use hydroxy_charts::connected;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::connected::Moment;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct, placed};

/// Plot width over height assumed for even timing and hit-testing; the plot stretches.
const ASPECT: f64 = 2.4;

/// How the pair of measures moved together over time: each moment is a dot at (x, y), and
/// a line joins them in time order, drawing itself from the first moment to the last. The
/// start is hollow, the end solid, and a few points carry their time. Hover or touch a dot
/// to read it; focus the plot and use Left/Right to step through time, Home/End to jump.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ConnectedScatterChart(
    /// The moments in time order: `Moment::new(label, x, y)`. NaN leaves a gap.
    #[prop(into)]
    data: Signal<Vec<Moment>>,
    /// What the chart shows, such as "Unemployment against inflation, 2008–2024". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names the horizontal measure, with its unit.
    #[prop(into)]
    x_label: String,
    /// Names the vertical measure, with its unit.
    #[prop(into)]
    y_label: String,
    /// Plot height in px, excluding the axis labels.
    #[prop(default = 300)]
    height: u16,
    /// How many points carry a time label, at most. The first and last always do.
    #[prop(default = 6)]
    time_labels: usize,
    /// Linear (default), Log or Symlog horizontal axis.
    #[prop(optional)]
    x_scale: ScaleKind,
    /// Linear (default), Log or Symlog vertical axis.
    #[prop(optional)]
    y_scale: ScaleKind,
    /// Formats horizontal values. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format_x: Option<Callback<f64, String>>,
    /// Formats vertical values. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format_y: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units on both axes.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let (fx, fy) = (formatter(format_x), formatter(format_y));
    let count = move || data.with(Vec::len);
    let layout = Memo::new(move |_| data.with(|d| annotations.with(|a| connected::layout(d, x_scale, y_scale, a, time_labels, ASPECT))));
    let active = RwSignal::new(None::<usize>);
    let names = StoredValue::new((x_label, y_label));

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let (x, y) = layout.with(|l| l.points.get(i).copied().flatten())?;
        let (xn, yn) = names.get_value();
        let rows = vec![TipRow::new(None, xn, plot::cell(Some(d.x), fx)), TipRow::new(None, yn, plot::cell(Some(d.y), fy))];
        Some(Tip { x, y, title: d.label, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|p| layout.with(|l| connected::nearest_point(&l.points, p, ASPECT, 0.12)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some());
    let ticks = Signal::derive(move || layout.with(|l| l.y_axis.ticks(fy)));
    let labels = Signal::derive(move || layout.with(|l| l.x_axis.ticks(fx)));
    let notes = placed(move |a| layout.with(|l| plot::place(a, false, |x| l.x_axis.scale.map(x), |v| l.y_axis.scale.map(v))), annotations);
    let head = Signal::derive(move || {
        let (xn, yn) = names.get_value();
        vec![String::new(), xn, yn]
    });
    let rows = Signal::derive(move || data.with(|d| d.iter().map(|d| vec![d.label.clone(), plot::cell(Some(d.x), fx), plot::cell(Some(d.y), fy)]).collect()));
    let mask_id = crate::id::next_id("connected-scatter-line");
    let marks = move || {
        let mask_id = mask_id.clone();
        let l = layout.get();
        let (xn, yn) = names.get_value();
        let last = l.points.iter().rposition(Option::is_some);
        let first = l.points.iter().position(Option::is_some);
        let grid = l.x_axis.gridlines.iter().map(|g| view! { <div data-part="x-gridline" style=format!("--p: {};", pct(g.fraction))></div> }).collect_view();
        let dots = l
            .points
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let (x, y) = (*p)?;
                let end = if Some(i) == last { Some("end") } else if Some(i) == first { Some("start") } else { None };
                let is_active = move || (active.get() == Some(i)).then_some("");
                let style = format!("--c: {x:.4}; --v: {y:.4}; --t: {:.4};", l.along[i]);
                Some(view! { <span data-part="dot" data-end=end data-active=is_active style=style></span> })
            })
            .collect_view();
        let times = l
            .labelled
            .iter()
            .filter_map(|&i| {
                let (x, y) = l.points[i]?;
                let text = data.with(|d| d.get(i).map(|d| d.label.clone()))?;
                let side = if x > 0.85 { "start" } else { "end" };
                let style = format!("--c: {x:.4}; --v: {y:.4}; --t: {:.4};", l.along[i]);
                Some(view! { <span data-part="point-label" data-side=side style=style>{text}</span> })
            })
            .collect_view();
        view! {
            <div data-part="x-grid" aria-hidden="true">{grid}</div>
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                <defs>
                    <mask id=mask_id.clone() maskUnits="userSpaceOnUse" x="-10" y="-10" width="120" height="120">
                        <path data-part="line-mask" d=l.path.clone() pathLength="1"></path>
                    </mask>
                </defs>
                <path data-part="line" d=l.path stroke=colour(0) mask=format!("url(#{mask_id})")></path>
            </svg>
            <div data-part="points" aria-hidden="true">{dots}{times}</div>
            <span data-part="axis-title" data-axis="y" aria-hidden="true">{yn}</span>
            <span data-part="axis-title" data-axis="x" aria-hidden="true">{xn}</span>
        }
    };

    view! {
        <Plot
            scope="connected-scatter-chart"
            label
            hint="Use the left and right arrow keys to step through time."
            height
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
