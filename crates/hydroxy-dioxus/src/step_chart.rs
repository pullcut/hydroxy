//! `StepChart`: values that hold, then jump, drawn as steps, with optional labelled
//! events. Geometry comes from `hydroxy_charts::step`.

use dioxus::prelude::*;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use hydroxy_charts::step;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::scale::ScaleKind;
pub use hydroxy_charts::step::{StepEvent, StepMode};

use crate::plot_kit::{Plot, formatter, pct};

/// Values that change in jumps (prices, rates, headcount, plan limits), one step line per
/// series, with labelled event markers. Hover or touch to read the values in force at a
/// point; focus the plot and use Left/Right to step, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn StepChart(
    /// One x label per point, such as dates.
    labels: ReadSignal<Vec<String>>,
    /// The series, one value per label. NaN leaves a gap. More than six fold into "Other".
    series: ReadSignal<Vec<Series>>,
    /// What the chart shows, such as "Bank Rate since 2020". Used as the accessible name.
    label: String,
    /// Where the jump falls: after each point (default), before it, or halfway.
    #[props(default)]
    mode: StepMode,
    /// Labelled moments, drawn as markers that fade in once the lines have drawn.
    #[props(default)]
    events: ReadSignal<Vec<StepEvent>>,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 240)]
    height: u16,
    /// Linear (default), Log or Symlog value axis.
    #[props(default)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a point index).
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = formatter(format);
    let names = labels();
    let n = names.len();
    let folded = plot::fold_series(&series(), 6);
    let marks_at = events();
    let notes = annotations();
    let layout = step::layout(&folded, n, mode, scale, &notes);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let event_at =
        |i: usize| marks_at.iter().filter(|e| e.at == i).map(|e| e.label.clone()).collect::<Vec<_>>().join(", ");

    let tip = current.map(|i| {
        let mut rows: Vec<TipRow> = folded
            .iter()
            .enumerate()
            .map(|(k, s)| TipRow::new(Some(colour(k)), s.name.clone(), plot::cell(s.values.get(i).copied(), fmt)))
            .collect();
        let note = event_at(i);
        if !note.is_empty() {
            rows.push(TipRow::new(None, "Event", note));
        }
        let y = layout.points.iter().filter_map(|p| p.get(i).copied().flatten()).fold(0.0, f64::max);
        Tip { x: layout.xs[i], y, title: names[i].clone(), rows }
    });
    let legend: Vec<Key> = if folded.len() < 2 {
        Vec::new()
    } else {
        folded.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect()
    };
    let has_events = !marks_at.is_empty();
    let mut head = vec![String::new()];
    head.extend(folded.iter().map(|s| s.name.clone()));
    if has_events {
        head.push("Event".into());
    }
    let rows: Vec<Vec<String>> = names
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let mut r = vec![l.clone()];
            r.extend(folded.iter().map(|s| plot::cell(s.values.get(i).copied(), fmt)));
            if has_events {
                r.push(event_at(i));
            }
            r
        })
        .collect();
    let xs = layout.xs.clone();
    let markers: Vec<(usize, f64, String)> =
        marks_at.iter().filter(|e| e.at < n).enumerate().map(|(k, e)| (k, layout.xs[e.at], e.label.clone())).collect();

    rsx! {
        Plot {
            scope: "step-chart",
            label,
            hint: "Use the left and right arrow keys to step through.",
            height,
            variant: mode.attr(),
            ticks: layout.axis.ticks(fmt),
            labels: plot::point_ticks(&names, &layout.xs, 8),
            annotations: plot::place(&notes, false, |x| plot::index_at(x, n), |v| layout.axis.scale.map(v)),
            legend,
            tip,
            head,
            rows,
            on_key: move |key: String| list_key(&key, active(), n).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, _)| step::held_index(&xs, x, mode));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                svg { "data-part": "marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    for (i , d) in layout.lines.iter().enumerate() {
                        path { key: "{i}", "data-part": "line", d: "{d}", stroke: colour(i), style: "--i: {i};" }
                    }
                }
                div { "data-part": "events", "aria-hidden": "true",
                    for (k , x , text) in markers {
                        div {
                            key: "{k}",
                            "data-part": "event",
                            "data-row": "{k % 2}",
                            "data-side": if x > 0.75 { "end" } else { "start" },
                            style: "--p: {pct(x)}; --i: {k};",
                            span { "data-part": "event-label", "{text}" }
                        }
                    }
                }
            },
            overlay: current.map(|i| {
                let x = layout.xs[i];
                rsx! {
                    div { "data-part": "crosshair", style: "--p: {pct(x)};" }
                    for (k , y) in layout.points.iter().map(|p| p.get(i).copied().flatten()).enumerate() {
                        if let Some(y) = y {
                            span { "data-part": "dot", style: "--c: {x:.4}; --v: {y:.4}; background: {colour(k)};" }
                        }
                    }
                }
            }),
        }
    }
}
