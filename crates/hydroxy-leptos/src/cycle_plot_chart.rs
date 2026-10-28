//! `<CyclePlotChart>`: one band per period of a cycle (month, weekday), each holding a
//! mini line across the cycles (years, weeks) and a flat line at that period's mean.
//! Geometry comes from `hydroxy_charts::cycle`.

use leptos::prelude::*;
use hydroxy_charts::cycle;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, pct, placed};

/// Seasonal shape and trend at once: the x axis is the periods of a cycle, and inside each
/// period a short line runs through that period's value in every cycle, oldest first, with
/// a flat line at their mean. Hover or touch to read a period; focus the plot and use
/// Left/Right to step through every point, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn CyclePlotChart(
    /// The periods along the axis, such as the months of the year.
    #[prop(into)]
    periods: Signal<Vec<String>>,
    /// One series per cycle (such as a year), oldest first, with a value per period. NaN leaves a gap.
    #[prop(into)]
    cycles: Signal<Vec<Series>>,
    /// What the chart shows, such as "Rainfall by month, 2021 to 2025". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px.
    #[prop(default = 260)]
    height: u16,
    /// Linear (default), Log or Symlog value axis. The axis fits the data rather than starting at zero.
    #[prop(optional)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a period index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let count = move || periods.with(Vec::len);
    let n_cycles = move || cycles.with(Vec::len);
    let layout = Memo::new(move |_| cycles.with(|c| annotations.with(|a| cycle::layout(c, count(), scale, a))));
    // Flattened as period * cycles + cycle, so Left/Right walk every point in reading order.
    let active = RwSignal::new(None::<usize>);
    let split = move |i: usize| (i / n_cycles().max(1), i % n_cycles().max(1));
    let tip = Signal::derive(move || {
        let (p, c) = split(active.get()?);
        let title = periods.with(|n| n.get(p).cloned())?;
        let (x, y) = layout.with(|l| l.points.get(p).and_then(|row| row.get(c).copied().flatten()))?;
        let mean = layout.with(|l| l.means.get(p).copied().flatten());
        let rows = cycles.with(|all| {
            all.iter()
                .enumerate()
                .map(|(k, s)| TipRow { active: k == c, ..TipRow::new(None, s.name.clone(), plot::cell(s.values.get(p).copied(), fmt)) })
                .chain(mean.map(|m| TipRow::new(Some("var(--text-2)".into()), "Mean", fmt(m.value))))
                .collect()
        });
        Some(Tip { x, y, title, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, y)| layout.with(|l| cycle::nearest(l, x, y))).map(|(p, c)| p * n_cycles() + c);
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| list_key(&key, active.get_untracked(), count() * n_cycles()).map(|next| active.set(next)).is_some());
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || periods.with(|n| plot::category_ticks(n, 12)));
    let notes = placed(move |a| layout.with(|l| plot::place(a, false, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v))), annotations);
    let legend = Signal::derive(move || {
        let span = cycles.with(|c| match (c.first(), c.last()) {
            (Some(a), Some(b)) if c.len() > 1 => format!("{} to {}", a.name, b.name),
            (Some(a), _) => a.name.clone(),
            _ => String::new(),
        });
        vec![Key::new(span, colour(0)), Key::new("Mean", "var(--text-2)")]
    });
    let head = Signal::derive(move || std::iter::once(String::new()).chain(cycles.with(|c| c.iter().map(|s| s.name.clone()).collect::<Vec<_>>())).chain(["Mean".to_string()]).collect());
    let rows = Signal::derive(move || {
        periods.with(|names| {
            cycles.with(|c| {
                layout.with(|l| {
                    names
                        .iter()
                        .enumerate()
                        .map(|(p, name)| {
                            std::iter::once(name.clone())
                                .chain(c.iter().map(|s| plot::cell(s.values.get(p).copied(), fmt)))
                                .chain([plot::cell(l.means.get(p).copied().flatten().map(|m| m.value), fmt)])
                                .collect()
                        })
                        .collect()
                })
            })
        })
    });
    let marks = move || {
        let l = layout.get();
        let means = l
            .means
            .iter()
            .enumerate()
            .filter_map(|(p, m)| m.map(|m| (p, m)))
            .map(|(p, m)| {
                let style = format!("left: {}; width: {}; bottom: {}; --i: {p};", pct(m.from), pct(m.to - m.from), pct(m.at));
                view! { <span data-part="mean" style=style></span> }
            })
            .collect_view();
        // The latest cycle gets a dot, so this year reads against the others.
        let latest = l
            .points
            .iter()
            .enumerate()
            .filter_map(|(p, row)| row.last().copied().flatten().map(|pt| (p, pt)))
            .map(|(p, (x, y))| view! { <span data-part="dot" style=format!("--c: {x:.4}; --v: {y:.4}; --i: {p}; background: {};", colour(0))></span> })
            .collect_view();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                <path data-part="line" d=l.lines stroke=colour(0)></path>
            </svg>
            <div data-part="points">{means}{latest}</div>
        }
    };
    let overlay = move || {
        let (p, c) = split(active.get()?);
        let (x, y) = layout.with(|l| l.points.get(p).and_then(|row| row.get(c).copied().flatten()))?;
        Some(view! { <span data-part="dot" style=format!("--c: {x:.4}; --v: {y:.4}; background: {};", colour(0))></span> })
    };

    view! {
        <Plot
            scope="cycle-plot-chart"
            label
            hint="Use the left and right arrow keys to step through every point."
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
