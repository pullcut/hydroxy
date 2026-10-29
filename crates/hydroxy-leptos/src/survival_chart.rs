//! `<SurvivalChart>`: Kaplan–Meier survival curves. Each group's curve steps down at every
//! event; ticks mark censored subjects; an optional band shows the Greenwood 95% interval
//! and a table under the axis counts who is still at risk. Geometry comes from
//! `hydroxy_charts::survival`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::line::nearest_index;
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{Key, Tick, Tip, TipRow, colour};
use hydroxy_charts::survival::{self, survival_at};
use leptos::prelude::*;

pub use hydroxy_charts::survival::{Observation, SurvivalGroup};

use crate::plot_kit::{Plot, formatter, pct};

/// A survival share as a whole percentage, such as "75%".
fn share(p: f64) -> String {
    format!("{:.0}%", p * 100.0)
}

/// The share still event-free over time, one step curve per group (up to six). Hover or touch
/// to read every group's estimate at that time; focus the plot and use Left/Right to step
/// through observed times, Home/End to jump, Escape to clear.
#[component]
pub fn SurvivalChart(
    /// The groups: `SurvivalGroup::new(name, [(time, event), …])`, where `event` is `false`
    /// for a censored subject.
    #[prop(into)]
    data: Signal<Vec<SurvivalGroup>>,
    /// What the chart shows, such as "Time to churn by plan". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// What time is measured in, for the tooltip and table, such as "Months".
    #[prop(default = "Time".into(), into)]
    time_label: String,
    /// Shade each curve's Greenwood 95% confidence interval.
    #[prop(optional)]
    show_ci: bool,
    /// Show the number at risk at each time tick under the axis.
    #[prop(optional)]
    risk_table: bool,
    /// Plot height in px, excluding the axis labels, legend and risk table.
    #[prop(default = 260)]
    height: u16,
    /// Formats times for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused time changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a time.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let layout = Memo::new(move |_| data.with(|d| survival::layout(d)));
    let positions =
        Memo::new(move |_| layout.with(|l| l.times.iter().map(|&t| l.x_axis.scale.map(t)).collect::<Vec<f64>>()));
    let count = move || layout.with(|l| l.times.len());
    let active = RwSignal::new(None::<usize>);
    let time_name = StoredValue::new(time_label);
    let estimate =
        move |steps: &[survival::Step], t: f64| survival_at(steps, t).map_or((1.0, (1.0, 1.0)), |s| (s.survival, s.ci));

    let tip = Signal::derive(move || {
        let i = active.get()?;
        layout.with(|l| {
            let t = *l.times.get(i)?;
            let rows = l
                .curves
                .iter()
                .enumerate()
                .map(|(g, c)| {
                    let (s, (lo, hi)) = estimate(&c.steps, t);
                    let value =
                        if show_ci { format!("{} ({}–{})", share(s), share(lo), share(hi)) } else { share(s) };
                    TipRow { active: l.curves.len() == 1, ..TipRow::new(Some(colour(g)), c.name.clone(), value) }
                })
                .collect();
            let y = l.curves.iter().map(|c| estimate(&c.steps, t).0).fold(0.0, f64::max);
            Some(Tip { x: l.x_axis.scale.map(t), y, title: format!("{} {}", time_name.get_value(), fmt(t)), rows })
        })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| positions.with(|xs| nearest_index(xs, x)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        list_key(&key, active.get_untracked(), count()).map(|next| active.set(next)).is_some()
    });
    let datum = Signal::derive(move || {
        let i = active.get()?;
        layout.with(|l| {
            let t = *l.times.get(i)?;
            let value = l.curves.first().map_or(1.0, |c| estimate(&c.steps, t).0);
            Some(EventDatum {
                series: 0,
                index: i,
                label: Some(format!("{} {}", time_name.get_value(), fmt(t))),
                value,
                node: None,
            })
        })
    });
    let ticks = Signal::derive(move || {
        layout.with(|l| {
            l.axis
                .gridlines
                .iter()
                .map(|g| Tick { at: g.fraction, text: share(g.value), baseline: g.value == 0.0 })
                .collect::<Vec<_>>()
        })
    });
    let labels = Signal::derive(move || layout.with(|l| l.x_axis.ticks(&fmt)));
    let legend = Signal::derive(move || {
        layout.with(|l| {
            let keys: Vec<Key> =
                l.curves.iter().enumerate().map(|(g, c)| Key::new(c.name.clone(), colour(g))).collect();
            if keys.len() > 1 { keys } else { Vec::new() }
        })
    });
    let head = Signal::derive(move || {
        ["Group", &time_name.get_value(), "At risk", "Events", "Censored", "Survival", "95% low", "95% high"]
            .map(String::from)
            .to_vec()
    });
    let rows = Signal::derive(move || {
        layout.with(|l| {
            l.curves
                .iter()
                .flat_map(|c| {
                    c.steps.iter().map(|s| {
                        vec![
                            c.name.clone(),
                            fmt(s.time),
                            s.at_risk.to_string(),
                            s.events.to_string(),
                            s.censored.to_string(),
                            share(s.survival),
                            share(s.ci.0),
                            share(s.ci.1),
                        ]
                    })
                })
                .collect()
        })
    });
    let marks = move || {
        let l = layout.get();
        let paths = l
            .curves
            .iter()
            .enumerate()
            .map(|(g, c)| {
                let band = (show_ci && !c.band.is_empty())
                    .then(|| view! { <path data-part="band" d=c.band.clone() fill=colour(g)></path> });
                view! {
                    {band}
                    <path data-part="line" d=c.line.clone() stroke=colour(g)></path>
                }
            })
            .collect_view();
        let censors = l
            .curves
            .iter()
            .enumerate()
            .flat_map(|(g, c)| {
                c.censors.iter().map(move |&(x, y)| {
                    let style = format!("--c: {x:.4}; --v: {y:.4}; background: {};", colour(g));
                    view! { <span data-part="censor" style=style></span> }
                })
            })
            .collect_view();
        view! {
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">{paths}</svg>
            <div data-part="points" aria-hidden="true">{censors}</div>
        }
    };
    let overlay = move || {
        let i = active.get()?;
        layout.with(|l| {
            let t = *l.times.get(i)?;
            let fx = l.x_axis.scale.map(t);
            let dots = l
                .curves
                .iter()
                .enumerate()
                .map(|(g, c)| {
                    let style =
                        format!("--c: {fx:.4}; --v: {:.4}; background: {};", estimate(&c.steps, t).0, colour(g));
                    view! { <span data-part="dot" style=style></span> }
                })
                .collect_view();
            Some(view! {
                <div data-part="crosshair" style=format!("--p: {};", pct(fx))></div>
                {dots}
            })
        })
    };
    let table = move || {
        risk_table.then(|| {
            let l = layout.get();
            let groups = l
                .curves
                .iter()
                .zip(&l.risk)
                .enumerate()
                .map(|(g, (c, counts))| {
                    let cells = l
                        .x_axis
                        .gridlines
                        .iter()
                        .zip(counts)
                        .map(|(tick, n)| view! { <span data-part="risk-count" style=format!("--p: {};", pct(tick.fraction))>{n.to_string()}</span> })
                        .collect_view();
                    view! {
                        <div data-part="risk-row">
                            <span data-part="risk-name" title=c.name.clone()>
                                <span data-part="swatch" style=format!("--swatch: {};", colour(g))></span>
                                {c.name.clone()}
                            </span>
                            <div data-part="risk-counts">{cells}</div>
                        </div>
                    }
                })
                .collect_view();
            view! {
                <div data-scope="survival-chart" data-part="risk-table" aria-hidden="true">
                    <div data-part="risk-title">"Number at risk"</div>
                    {groups}
                </div>
            }
        })
    };

    view! {
        <Plot
            scope="survival-chart"
            label
            hint="Use the left and right arrow keys to step through times."
            height
            variant=if show_ci { "ci" } else { "" }
            ticks
            labels
            annotations=Signal::derive(Vec::new)
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
        {table}
    }
}
