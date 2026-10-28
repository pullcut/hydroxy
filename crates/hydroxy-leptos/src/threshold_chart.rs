//! `<ThresholdChart>`: one line coloured differently above and below a threshold.
//! Geometry comes from `hydroxy_charts::split::threshold`; the line and its fill are drawn
//! twice and clipped at the threshold.

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

/// A line read against a threshold: above it in the first chart colour, below it in the
/// second, with the gap to the threshold shaded. Hover or touch to read a point; focus the
/// plot and use Left/Right to step, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ThresholdChart(
    /// The values in x order. NaN leaves a gap.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// The value the line is judged against.
    #[prop(into)]
    threshold: Signal<f64>,
    /// What the chart shows, such as "Server error rate". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names the threshold line, tooltip row and table column.
    #[prop(default = "Threshold".into(), into)]
    threshold_label: String,
    /// Legend name for values above the threshold.
    #[prop(default = "Above".into(), into)]
    above_label: String,
    /// Legend name for values below the threshold.
    #[prop(default = "Below".into(), into)]
    below_label: String,
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
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || data.with(Vec::len);
    let layout = Memo::new(move |_| {
        let values: Vec<f64> = data.with(|d| d.iter().map(|d| d.value).collect());
        annotations.with(|a| split::threshold(&values, threshold.get(), scale, curve, a))
    });
    let active = RwSignal::new(None::<usize>);
    let names = StoredValue::new((threshold_label, above_label, below_label));

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let limit = threshold.get();
        let (x, y) = layout.with(|l| (l.xs[i], l.points[i].unwrap_or(l.threshold)));
        let above = d.value >= limit;
        let (tname, _, _) = names.get_value();
        let gap = d.value - limit;
        let sign = if gap >= 0.0 { "+" } else { "−" };
        let rows = vec![
            TipRow {
                active: true,
                ..TipRow::new(Some(colour(usize::from(!above))), "", plot::cell(Some(d.value), fmt))
            },
            TipRow::new(None, tname, fmt(limit)),
            TipRow::new(
                None,
                "Difference",
                if d.value.is_finite() { format!("{sign}{}", fmt(gap.abs())) } else { "–".into() },
            ),
        ];
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
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
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
        let (_, above, below) = names.get_value();
        vec![Key::new(above, colour(0)), Key::new(below, colour(1))]
    });
    let head = Signal::derive(move || vec![String::new(), "Value".to_string(), names.get_value().0]);
    let rows = Signal::derive(move || {
        let t = threshold.get();
        data.with(|d| d.iter().map(|d| vec![d.label.clone(), plot::cell(Some(d.value), fmt), fmt(t)]).collect())
    });
    let marks = move || {
        let l = layout.get();
        let t = l.threshold;
        // The same drawing twice: once showing only what is above the threshold, once below.
        let half = |i: usize, clip: String| {
            view! {
                <div data-part="clip" data-side=if i == 0 { "above" } else { "below" } style=clip>
                    <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                        <path data-part="area" d=l.area.clone() fill=colour(i)></path>
                        <path data-part="line" d=l.line.clone() stroke=colour(i)></path>
                    </svg>
                </div>
            }
        };
        view! {
            {half(0, format!("clip-path: inset(0 0 {} 0);", pct(t)))}
            {half(1, format!("clip-path: inset({} 0 0 0);", pct(1.0 - t)))}
            <div data-part="threshold" style=format!("--p: {};", pct(t))>
                <span>{names.get_value().0}</span>
            </div>
        }
    };
    let overlay = move || {
        active.get().and_then(|i| {
            let (x, y) = layout.with(|l| Some((l.xs[i], l.points[i]?)))?;
            let above = data.with(|d| d.get(i).is_some_and(|d| d.value >= threshold.get()));
            Some(view! {
                <div data-part="crosshair" style=format!("--p: {};", pct(x))></div>
                <span data-part="dot" style=format!("--c: {x:.4}; --v: {y:.4}; background: {};", colour(usize::from(!above)))></span>
            })
        })
    };

    view! {
        <Plot
            scope="threshold-chart"
            label
            hint="Use the left and right arrow keys to step through."
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
        />
    }
}
