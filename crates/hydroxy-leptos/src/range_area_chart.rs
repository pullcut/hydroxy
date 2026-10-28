//! `<RangeAreaChart>`: a band from each point's low to its high, such as daily low and
//! high temperatures, with an optional line through a middle value. Geometry comes from
//! `hydroxy_charts::interval::band`.

use leptos::prelude::*;
use hydroxy_charts::interval;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::Interval;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct, placed};

/// The band's legend swatch: the series hue, faded as the band is.
fn band_colour() -> String {
    format!("color-mix(in oklab, {} 35%, transparent)", colour(0))
}

/// A min–max band per x position, revealed left to right, with an optional middle line
/// (a mean or median). Hover or touch to read the low, high and spread; focus the plot and
/// use Left/Right to step, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RangeAreaChart(
    /// One range per x position: `Interval::new(label, mid, low, high)`. A NaN mid draws no
    /// line there; a NaN low or high leaves a gap in the band.
    #[prop(into)]
    data: Signal<Vec<Interval>>,
    /// What the chart shows, such as "Daily temperature range". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names the band in the legend, tooltip and table.
    #[prop(default = "Range".into(), into)]
    range_label: String,
    /// Names the middle line in the legend, tooltip and table.
    #[prop(default = "Mean".into(), into)]
    mid_label: String,
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
        data.with(|d| {
            let pick = |f: fn(&Interval) -> f64| d.iter().map(f).collect::<Vec<f64>>();
            annotations.with(|a| interval::band(&pick(|d| d.value), &pick(|d| d.low), &pick(|d| d.high), scale, curve, a))
        })
    });
    let has_mid = Memo::new(move |_| data.with(|d| d.iter().any(|d| d.value.is_finite())));
    let active = RwSignal::new(None::<usize>);
    let names = StoredValue::new((range_label, mid_label));
    let at = move |v: f64| v.is_finite().then(|| layout.with(|l| l.axis.at(v)));

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let (rname, mname) = names.get_value();
        let mut rows = vec![
            TipRow::new(Some(band_colour()), "High", plot::cell(Some(d.high), fmt)),
            TipRow::new(Some(band_colour()), "Low", plot::cell(Some(d.low), fmt)),
        ];
        if d.value.is_finite() {
            rows.push(TipRow::new(Some(colour(0)), mname, plot::cell(Some(d.value), fmt)));
        }
        rows.push(TipRow { active: true, ..TipRow::new(None, rname, plot::cell(Some(d.high - d.low), fmt)) });
        let x = layout.with(|l| l.xs[i]);
        Some(Tip { x, y: at(d.high).or(at(d.value)).unwrap_or(0.0), title: d.label, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| layout.with(|l| nearest_index(&l.xs, x)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some());
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || data.with(|d| layout.with(|l| plot::point_ticks(&d.iter().map(|d| d.label.clone()).collect::<Vec<_>>(), &l.xs, 8))));
    let notes = placed(move |a| layout.with(|l| plot::place(a, false, |x| plot::index_at(x, count()), |v| l.axis.scale.map(v))), annotations);
    let legend = Signal::derive(move || {
        let (rname, mname) = names.get_value();
        if has_mid.get() { vec![Key::new(rname, band_colour()), Key::new(mname, colour(0))] } else { Vec::new() }
    });
    let head = Signal::derive(move || {
        let (rname, mname) = names.get_value();
        let mut h = vec![String::new(), "Low".to_string(), "High".to_string()];
        if has_mid.get() {
            h.push(mname);
        }
        h.push(rname);
        h
    });
    let rows = Signal::derive(move || {
        let mid = has_mid.get();
        data.with(|d| {
            d.iter()
                .map(|d| {
                    let mut r = vec![d.label.clone(), plot::cell(Some(d.low), fmt), plot::cell(Some(d.high), fmt)];
                    if mid {
                        r.push(plot::cell(Some(d.value), fmt));
                    }
                    r.push(plot::cell(Some(d.high - d.low), fmt));
                    r
                })
                .collect()
        })
    });
    let marks = move || {
        let l = layout.get();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                <path data-part="band" d=l.band fill=colour(0)></path>
                {has_mid.get().then(|| view! { <path data-part="line" d=l.line stroke=colour(0)></path> })}
            </svg>
        }
    };
    let overlay = move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let x = layout.with(|l| l.xs[i]);
        let dot = |v: f64, part: &'static str| at(v).map(|y| view! { <span data-part="dot" data-kind=part style=format!("--c: {x:.4}; --v: {y:.4};")></span> });
        Some(view! {
            <div data-part="crosshair" style=format!("--p: {};", pct(x))></div>
            {at(d.low).zip(at(d.high)).map(|(lo, hi)| view! { <div data-part="span" style=format!("--c: {x:.4}; --a: {lo:.4}; --b: {hi:.4};")></div> })}
            {dot(d.high, "high")}
            {dot(d.low, "low")}
            {dot(d.value, "mid")}
        })
    };

    view! {
        <Plot
            scope="range-area-chart"
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
