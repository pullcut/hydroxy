//! `<StepChart>`: values that hold, then jump, drawn as steps, with optional labelled
//! events. Geometry comes from `hydroxy_charts::step`.

use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use hydroxy_charts::step;
use leptos::prelude::*;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::scale::ScaleKind;
pub use hydroxy_charts::step::{StepEvent, StepMode};

use crate::plot_kit::{Plot, formatter, pct, placed};

/// Values that change in jumps (prices, rates, headcount, plan limits), one step line per
/// series, with labelled event markers. Hover or touch to read the values in force at a
/// point; focus the plot and use Left/Right to step, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn StepChart(
    /// One x label per point, such as dates.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// The series, one value per label. NaN leaves a gap. More than six fold into "Other".
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// What the chart shows, such as "Bank Rate since 2020". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Where the jump falls: after each point (default), before it, or halfway.
    #[prop(optional)]
    mode: StepMode,
    /// Labelled moments, drawn as markers that fade in once the lines have drawn.
    #[prop(optional, into)]
    events: Signal<Vec<StepEvent>>,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 240)]
    height: u16,
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
    let count = move || labels.with(Vec::len);
    let folded = Memo::new(move |_| series.with(|s| plot::fold_series(s, 6)));
    let layout = Memo::new(move |_| folded.with(|s| annotations.with(|a| step::layout(s, count(), mode, scale, a))));
    let active = RwSignal::new(None::<usize>);
    let event_at = move |i: usize| {
        events.with(|e| e.iter().filter(|e| e.at == i).map(|e| e.label.clone()).collect::<Vec<_>>().join(", "))
    };

    let tip = Signal::derive(move || {
        let i = active.get()?;
        let title = labels.with(|l| l.get(i).cloned())?;
        let mut rows: Vec<TipRow> = folded.with(|s| {
            s.iter()
                .enumerate()
                .map(|(k, s)| TipRow::new(Some(colour(k)), s.name.clone(), plot::cell(s.values.get(i).copied(), fmt)))
                .collect()
        });
        let note = event_at(i);
        if !note.is_empty() {
            rows.push(TipRow::new(None, "Event", note));
        }
        let (x, y) =
            layout.with(|l| (l.xs[i], l.points.iter().filter_map(|p| p.get(i).copied().flatten()).fold(0.0, f64::max)));
        Some(Tip { x, y, title, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| layout.with(|l| step::held_index(&l.xs, x, mode)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let x_labels = Signal::derive(move || labels.with(|n| layout.with(|l| plot::point_ticks(n, &l.xs, 8))));
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, false, |x| plot::index_at(x, count()), |v| l.axis.scale.map(v))),
        annotations,
    );
    let legend = Signal::derive(move || {
        folded.with(|s| {
            if s.len() < 2 {
                Vec::new()
            } else {
                s.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect()
            }
        })
    });
    let has_events = Signal::derive(move || events.with(|e| !e.is_empty()));
    let head = Signal::derive(move || {
        let mut h = vec![String::new()];
        folded.with(|s| h.extend(s.iter().map(|s| s.name.clone())));
        if has_events.get() {
            h.push("Event".into());
        }
        h
    });
    let rows = Signal::derive(move || {
        labels.with(|labels| {
            folded.with(|s| {
                labels
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        let mut r = vec![l.clone()];
                        r.extend(s.iter().map(|s| plot::cell(s.values.get(i).copied(), fmt)));
                        if has_events.get() {
                            r.push(event_at(i));
                        }
                        r
                    })
                    .collect()
            })
        })
    });
    let marks = move || {
        let l = layout.get();
        let n = l.xs.len();
        let lines = l
            .lines
            .into_iter()
            .enumerate()
            .map(|(i, d)| view! { <path data-part="line" d=d stroke=colour(i) style=format!("--i: {i};")></path> })
            .collect_view();
        let markers = events
            .get()
            .into_iter()
            .filter(|e| e.at < n)
            .enumerate()
            .map(|(k, e)| {
                let x = l.xs[e.at];
                let side = if x > 0.75 { "end" } else { "start" };
                view! {
                    <div data-part="event" data-row=k % 2 data-side=side style=format!("--p: {}; --i: {k};", pct(x))>
                        <span data-part="event-label">{e.label}</span>
                    </div>
                }
            })
            .collect_view();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">{lines}</svg>
            <div data-part="events" aria-hidden="true">{markers}</div>
        }
    };
    let overlay = move || {
        let i = active.get()?;
        let (x, dots) =
            layout.with(|l| (l.xs[i], l.points.iter().map(|p| p.get(i).copied().flatten()).collect::<Vec<_>>()));
        let dots = dots
            .into_iter()
            .enumerate()
            .filter_map(|(k, y)| y.map(|y| view! { <span data-part="dot" style=format!("--c: {x:.4}; --v: {y:.4}; background: {};", colour(k))></span> }))
            .collect_view();
        Some(view! {
            <div data-part="crosshair" style=format!("--p: {};", pct(x))></div>
            {dots}
        })
    };

    view! {
        <Plot
            scope="step-chart"
            label
            hint=hydroxy_charts::nav::HINT_STEP_LR
            height
            variant=mode.attr()
            ticks
            labels=x_labels
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
