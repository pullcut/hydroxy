//! `<ConfidenceBandChart>`: an estimate as a line, with a shaded band from its low to its
//! high bound. Geometry comes from `hydroxy_charts::interval::band`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::interval;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::{HINT_STEP_LR, list_key};
use hydroxy_charts::plot::{self, Key, Tip, TipRow};
use leptos::prelude::*;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::interval::Interval;
pub use hydroxy_charts::line::Curve;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, colour, formatter, pct, placed};

/// A line with its uncertainty: the band shows where the true value probably lies. Hover or
/// touch to read a point and its bounds; focus the plot and use Left/Right to step,
/// Home/End to jump, Escape to clear.
#[component]
pub fn ConfidenceBandChart(
    /// One estimate per x position, `Interval::new(label, value, low, high)`. NaN leaves a gap.
    #[prop(into)]
    data: Signal<Vec<Interval>>,
    /// What the chart shows, such as "Daily conversion rate". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names the line in the legend, tooltip and table.
    #[prop(default = "Estimate".into(), into)]
    value_label: String,
    /// Names the band in the legend, tooltip and table.
    #[prop(default = "95% interval".into(), into)]
    band_label: String,
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
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || data.with(Vec::len);
    let layout = Memo::new(move |_| {
        data.with(|d| {
            let pick = |f: fn(&Interval) -> f64| d.iter().map(f).collect::<Vec<f64>>();
            annotations
                .with(|a| interval::band(&pick(|d| d.value), &pick(|d| d.low), &pick(|d| d.high), scale, curve, a))
        })
    });
    let active = RwSignal::new(None::<usize>);
    let names = StoredValue::new((value_label, band_label));
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            data.with(|d| d.get(i).map(|v| EventDatum::category(i, v.label.clone(), v.value)))
        })
    });
    let range = move |d: &Interval| format!("{} – {}", plot::cell(Some(d.low), &fmt), plot::cell(Some(d.high), &fmt));

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let d = data.with(|d| d.get(i).cloned())?;
        let (x, y) = layout.with(|l| (l.xs[i], l.points[i].unwrap_or(0.0)));
        let (vname, bname) = names.get_value();
        let rows = vec![
            TipRow { active: true, ..TipRow::new(Some(colour(0)), vname, plot::cell(Some(d.value), &fmt)) },
            TipRow::new(Some(colour(0)), bname, range(&d)),
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
        let (vname, bname) = names.get_value();
        vec![
            Key::new(vname, colour(0)),
            Key::new(bname, format!("color-mix(in oklab, {} 30%, transparent)", colour(0))),
        ]
    });
    let head = Signal::derive(move || {
        let (vname, _) = names.get_value();
        vec![String::new(), vname, "Low".to_string(), "High".to_string()]
    });
    let rows = Signal::derive(move || {
        data.with(|d| {
            d.iter()
                .map(|d| {
                    vec![
                        d.label.clone(),
                        plot::cell(Some(d.value), &fmt),
                        plot::cell(Some(d.low), &fmt),
                        plot::cell(Some(d.high), &fmt),
                    ]
                })
                .collect()
        })
    });
    let marks = move || {
        let l = layout.get();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                <path data-part="band" d=l.band fill=colour(0)></path>
                <path data-part="line" d=l.line stroke=colour(0)></path>
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
            scope="confidence-band-chart"
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
            datum
            on_hover
            on_select
            marks
            overlay
        />
    }
}
