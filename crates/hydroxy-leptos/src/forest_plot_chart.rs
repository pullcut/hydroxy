//! `<ForestPlotChart>`: study estimates with confidence intervals on a shared axis, the
//! line of no effect and a pooled diamond, beside columns of the numbers.
//! Geometry comes from `hydroxy_charts::forest`.

use hydroxy_charts::forest::{self, Mark};
use hydroxy_charts::nav;
use leptos::prelude::*;

pub use hydroxy_charts::forest::Study;

use crate::plot_shell::pct;

/// The text for an estimate and its interval: "0.80 [0.61, 1.05]".
pub(crate) fn interval_text(s: &Study, fmt: impl Fn(f64) -> String) -> String {
    if [s.estimate, s.low, s.high].iter().all(|v| v.is_finite()) {
        format!("{} [{}, {}]", fmt(s.estimate), fmt(s.low.min(s.high)), fmt(s.low.max(s.high)))
    } else {
        "No data".to_string()
    }
}

/// The interval's inline style: its span, with `--m` where the estimate sits inside it.
pub(crate) fn whisker_style(m: Mark) -> String {
    let at = if m.high > m.low { (m.at - m.low) / (m.high - m.low) } else { 0.5 };
    format!("left: {}; width: {}; --m: {};", pct(m.low), pct(m.high - m.low), pct(at))
}

/// The pooled diamond's inline style: its span, pointed at the estimate.
pub(crate) fn diamond_style(m: Mark) -> String {
    let at = if m.high > m.low { (m.at - m.low) / (m.high - m.low) } else { 0.5 };
    format!("{} clip-path: polygon(0 50%, {p} 0, 100% 50%, {p} 100%);", whisker_style(m), p = pct(at))
}

/// A meta-analysis at a glance: one row per study with its estimate as a square (sized by
/// weight) and its confidence interval as a whisker, a vertical line of no effect, and the
/// pooled estimate as a diamond, with the study, estimate and interval, and weight in
/// columns alongside. Ratios read on a log axis. Hover or touch a row, or focus the plot
/// and use the arrow keys to step through, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn ForestPlotChart(
    /// The studies, top to bottom: `Study::new(label, estimate, (low, high), weight)`.
    #[prop(into)]
    studies: Signal<Vec<Study>>,
    /// The pooled estimate, drawn as a diamond under the studies. Its weight is ignored.
    #[prop(optional, into)]
    pooled: Signal<Option<Study>>,
    /// What the chart shows, such as "Statin therapy and major cardiac events". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Name of the effect measure, such as "Odds ratio". Heads the plot column.
    #[prop(into)]
    measure: String,
    /// Put ratios on a log axis with the line of no effect at 1. Otherwise it sits at 0.
    #[prop(optional)]
    log: bool,
    /// What each side of the line of no effect means, such as ("Favours statin", "Favours placebo").
    #[prop(optional)]
    favours: Option<(String, String)>,
    /// Confidence level shown in the column heading, in percent.
    #[prop(default = 95)]
    level: u8,
    /// Formats estimates and limits. Default: two decimal places.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format!("{v:.2}"), |f| f.run(v));
    let layout = Memo::new(move |_| studies.with(|s| pooled.with(|p| forest::layout(s, p.as_ref(), log))));
    let count = move || studies.with(Vec::len) + usize::from(pooled.with(Option::is_some));
    let active = RwSignal::new(None::<usize>);
    // Row `i`: a study, or the pooled result after them all.
    let row = move |i: usize| -> Option<(Study, Option<Mark>, bool)> {
        let n = studies.with(Vec::len);
        if i < n {
            Some((studies.with(|s| s[i].clone()), layout.with(|l| l.rows[i]), false))
        } else {
            pooled.get().map(|p| (p, layout.with(|l| l.pooled), true))
        }
    };
    let text = move |i: usize| {
        row(i)
            .map(|(s, m, _)| {
                format!(
                    "{}: {}, weight {}",
                    s.label,
                    interval_text(&s, fmt),
                    m.map_or_else(|| "–".to_string(), |m| format!("{:.1}%", m.share * 100.0))
                )
            })
            .unwrap_or_default()
    };
    let aria = format!("{label}. Use the arrow keys to step through the studies.");
    let ci_head = format!("{measure} [{level}% CI]");
    let ci_table = ci_head.clone();
    let (left_note, right_note) = favours.unzip();

    let line = move |i: usize| {
        let (s, m, is_pooled) = row(i)?;
        let weight = m.map_or_else(
            || "–".to_string(),
            |m| if is_pooled { "100%".to_string() } else { format!("{:.1}%", m.share * 100.0) },
        );
        let mark = m.map(|m| {
            if is_pooled {
                view! { <span data-part="diamond" style=diamond_style(m)></span> }.into_any()
            } else {
                let side = format!("--s: {:.3};", m.size);
                view! {
                    <span data-part="whisker" style=whisker_style(m)></span>
                    <span data-part="square" style=format!("left: {}; {side}", pct(m.at))></span>
                }
                .into_any()
            }
        });
        let weight_tip = weight.clone();
        let (name, estimate) = (s.label.clone(), interval_text(&s, fmt));
        let tip = move || {
            if active.get() != Some(i) {
                return None;
            }
            let m = m?;
            let s = s.clone();
            Some(view! {
                    <div data-part="tooltip" data-side={if m.at > 0.5 { "left" } else { "right" }} style=format!("left: {};", pct(m.at)) aria-hidden="true">
                        <div data-part="tooltip-label">{s.label}</div>
                        <ul>
                            <li data-part="tooltip-row"><span data-part="tooltip-name">"Estimate"</span><span data-part="tooltip-value">{fmt(s.estimate)}</span></li>
                            <li data-part="tooltip-row"><span data-part="tooltip-name">"Interval"</span><span data-part="tooltip-value">{format!("{} to {}", fmt(s.low.min(s.high)), fmt(s.low.max(s.high)))}</span></li>
                            <li data-part="tooltip-row"><span data-part="tooltip-name">"Weight"</span><span data-part="tooltip-value">{weight_tip.clone()}</span></li>
                        </ul>
                    </div>
            })
        };
        let on_enter = move |_| active.set(Some(i));
        Some(view! {
            <div
                data-part="row"
                data-pooled=is_pooled.then_some("")
                data-active=move || (active.get() == Some(i)).then_some("")
                style=format!("--i: {}; grid-row: {};", i.min(30), i + 2)
                on:pointerenter=on_enter
                on:pointerdown=on_enter
            >
                <span data-part="study">{name}</span>
                <span data-part="interval">{mark}{tip}</span>
                <span data-part="estimate">{estimate}</span>
                <span data-part="weight">{weight}</span>
            </div>
        })
    };

    view! {
        <figure data-scope="forest-plot-chart" data-part="root" data-active=move || active.get().map(|_| "")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=aria
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let key = match ev.key().as_str() {
                        "ArrowDown" => "ArrowRight".to_string(),
                        "ArrowUp" => "ArrowLeft".to_string(),
                        k => k.to_string(),
                    };
                    if let Some(next) = nav::list_key(&key, active.get_untracked(), count()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <div data-part="head" aria-hidden="true">
                    <span data-part="study">"Study"</span>
                    <span data-part="interval">{measure.clone()}</span>
                    <span data-part="estimate">{ci_head}</span>
                    <span data-part="weight">"Weight"</span>
                </div>
                <div data-part="guides" aria-hidden="true" style=move || format!("grid-row: 2 / span {};", count().max(1))>
                    {move || layout.get().ticks.into_iter().map(|t| view! { <span data-part="gridline" style=format!("left: {};", pct(t.at))></span> }).collect_view()}
                    <span data-part="null" style=move || format!("left: {};", pct(layout.with(|l| l.null)))></span>
                </div>
                {move || (0..count()).filter_map(line).collect_view()}
                <div data-part="axis" aria-hidden="true" style=move || format!("grid-row: {};", count() + 2)>
                    <span data-part="ticks">
                        {move || layout.get().ticks.into_iter().map(|t| view! { <span data-part="tick" style=format!("left: {};", pct(t.at))>{fmt(t.value)}</span> }).collect_view()}
                    </span>
                    {left_note.map(|l| view! {
                        <span data-part="favours" data-side="left" style=move || format!("--null: {};", pct(layout.with(|l| l.null)))>{format!("← {l}")}</span>
                    })}
                    {right_note.map(|r| view! {
                        <span data-part="favours" data-side="right" style=move || format!("--null: {};", pct(layout.with(|l| l.null)))>{format!("{r} →")}</span>
                    })}
                </div>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(text).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr><th scope="col">"Study"</th><th scope="col">{ci_table}</th><th scope="col">"Weight"</th></tr>
                </thead>
                <tbody>
                    {move || (0..count()).filter_map(|i| row(i).map(|(s, m, p)| view! {
                        <tr>
                            <th scope="row">{s.label.clone()}</th>
                            <td>{interval_text(&s, fmt)}</td>
                            <td>{m.map_or_else(|| "–".to_string(), |m| if p { "100%".to_string() } else { format!("{:.1}%", m.share * 100.0) })}</td>
                        </tr>
                    })).collect_view()}
                </tbody>
            </table>
        </figure>
    }
}
