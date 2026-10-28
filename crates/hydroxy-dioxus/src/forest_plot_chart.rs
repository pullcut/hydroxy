//! `ForestPlotChart`: study estimates with confidence intervals on a shared axis, the
//! line of no effect and a pooled diamond, beside columns of the numbers.
//! Geometry comes from `hydroxy_charts::forest`.

use dioxus::prelude::*;
use hydroxy_charts::forest::{self, Mark};
use hydroxy_charts::nav;

pub use hydroxy_charts::forest::Study;

use crate::plot_shell::pct;

/// The text for an estimate and its interval: "0.80 [0.61, 1.05]".
fn interval_text(s: &Study, fmt: impl Fn(f64) -> String) -> String {
    if [s.estimate, s.low, s.high].iter().all(|v| v.is_finite()) {
        format!("{} [{}, {}]", fmt(s.estimate), fmt(s.low.min(s.high)), fmt(s.low.max(s.high)))
    } else {
        "No data".to_string()
    }
}

/// The interval's inline style: its span, with `--m` where the estimate sits inside it.
fn whisker_style(m: Mark) -> String {
    let at = if m.high > m.low { (m.at - m.low) / (m.high - m.low) } else { 0.5 };
    format!("left: {}; width: {}; --m: {};", pct(m.low), pct(m.high - m.low), pct(at))
}

/// The pooled diamond's inline style: its span, pointed at the estimate.
fn diamond_style(m: Mark) -> String {
    let at = if m.high > m.low { (m.at - m.low) / (m.high - m.low) } else { 0.5 };
    format!("{} clip-path: polygon(0 50%, {p} 0, 100% 50%, {p} 100%);", whisker_style(m), p = pct(at))
}

/// A meta-analysis at a glance: one row per study with its estimate as a square (sized by
/// weight) and its confidence interval as a whisker, a vertical line of no effect, and the
/// pooled estimate as a diamond, with the study, estimate and interval, and weight in
/// columns alongside. Ratios read on a log axis. Hover or touch a row, or focus the plot
/// and use the arrow keys to step through, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ForestPlotChart(
    /// The studies, top to bottom: `Study::new(label, estimate, (low, high), weight)`.
    studies: ReadSignal<Vec<Study>>,
    /// The pooled estimate, drawn as a diamond under the studies. Its weight is ignored.
    #[props(default)]
    pooled: ReadSignal<Option<Study>>,
    /// What the chart shows, such as "Statin therapy and major cardiac events". Used as the accessible name.
    label: String,
    /// Name of the effect measure, such as "Odds ratio". Heads the plot column.
    measure: String,
    /// Put ratios on a log axis with the line of no effect at 1. Otherwise it sits at 0.
    #[props(default)]
    log: bool,
    /// What each side of the line of no effect means, such as ("Favours statin", "Favours placebo").
    favours: Option<(String, String)>,
    /// Confidence level shown in the column heading, in percent.
    #[props(default = 95)]
    level: u8,
    /// Formats estimates and limits. Default: two decimal places.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format!("{v:.2}"), |f| f.call(v));
    let l = use_memo(use_reactive((&log,), move |(log,)| forest::layout(&studies.read(), pooled.read().as_ref(), log)));
    let l = l.read();
    let mut active = use_signal(|| None::<usize>);
    // Rows: each study, then the pooled result.
    let mut rows: Vec<(Study, Option<Mark>, bool)> = studies().into_iter().zip(l.rows.iter().copied()).map(|(s, m)| (s, m, false)).collect();
    if let Some(p) = pooled() {
        rows.push((p, l.pooled, true));
    }
    let count = rows.len();
    let current = active().filter(|&i| i < count);
    let weight = |m: Option<Mark>, pooled: bool| m.map_or_else(|| "–".to_string(), |m| if pooled { "100%".to_string() } else { format!("{:.1}%", m.share * 100.0) });
    let text = current.map(|i| format!("{}: {}, weight {}", rows[i].0.label, interval_text(&rows[i].0, fmt), weight(rows[i].1, rows[i].2))).unwrap_or_default();
    let null = pct(l.null);
    let (left_note, right_note) = favours.unzip();

    rsx! {
        figure { "data-scope": "forest-plot-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the studies.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    let key = match e.key().to_string().as_str() {
                        "ArrowDown" => "ArrowRight".to_string(),
                        "ArrowUp" => "ArrowLeft".to_string(),
                        k => k.to_string(),
                    };
                    if let Some(next) = nav::list_key(&key, active(), count) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                div { "data-part": "head", "aria-hidden": "true",
                    span { "data-part": "study", "Study" }
                    span { "data-part": "interval", "{measure}" }
                    span { "data-part": "estimate", "{measure} [{level}% CI]" }
                    span { "data-part": "weight", "Weight" }
                }
                div { "data-part": "guides", "aria-hidden": "true", style: "grid-row: 2 / span {count.max(1)};",
                    for t in l.ticks.iter() {
                        span { "data-part": "gridline", style: "left: {pct(t.at)};" }
                    }
                    span { "data-part": "null", style: "left: {null};" }
                }
                for (i , (s , m , is_pooled)) in rows.iter().enumerate() {
                    div {
                        key: "{i}",
                        "data-part": "row",
                        "data-pooled": is_pooled.then_some(""),
                        "data-active": (current == Some(i)).then_some(""),
                        style: "--i: {i.min(30)}; grid-row: {i + 2};",
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerdown: move |_| active.set(Some(i)),
                        span { "data-part": "study", "{s.label}" }
                        span { "data-part": "interval",
                            if let Some(m) = m {
                                if *is_pooled {
                                    span { "data-part": "diamond", style: diamond_style(*m) }
                                } else {
                                    span { "data-part": "whisker", style: whisker_style(*m) }
                                    span { "data-part": "square", style: "left: {pct(m.at)}; --s: {m.size:.3};" }
                                }
                                if current == Some(i) {
                                    div {
                                        "data-part": "tooltip",
                                        "data-side": if m.at > 0.5 { "left" } else { "right" },
                                        style: "left: {pct(m.at)};",
                                        "aria-hidden": "true",
                                        div { "data-part": "tooltip-label", "{s.label}" }
                                        ul {
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "tooltip-name", "Estimate" }
                                                span { "data-part": "tooltip-value", {fmt(s.estimate)} }
                                            }
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "tooltip-name", "Interval" }
                                                span { "data-part": "tooltip-value", "{fmt(s.low.min(s.high))} to {fmt(s.low.max(s.high))}" }
                                            }
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "tooltip-name", "Weight" }
                                                span { "data-part": "tooltip-value", {weight(Some(*m), *is_pooled)} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        span { "data-part": "estimate", {interval_text(s, fmt)} }
                        span { "data-part": "weight", {weight(*m, *is_pooled)} }
                    }
                }
                div { "data-part": "axis", "aria-hidden": "true", style: "grid-row: {count + 2};",
                    span { "data-part": "ticks",
                        for t in l.ticks.iter() {
                            span { "data-part": "tick", style: "left: {pct(t.at)};", {fmt(t.value)} }
                        }
                    }
                    if let Some(n) = left_note {
                        span { "data-part": "favours", "data-side": "left", style: "--null: {null};", "← {n}" }
                    }
                    if let Some(n) = right_note {
                        span { "data-part": "favours", "data-side": "right", style: "--null: {null};", "{n} →" }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{text}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Study" }
                        th { scope: "col", "{measure} [{level}% CI]" }
                        th { scope: "col", "Weight" }
                    }
                }
                tbody {
                    for (s , m , p) in rows.iter() {
                        tr {
                            th { scope: "row", "{s.label}" }
                            td { {interval_text(s, fmt)} }
                            td { {weight(*m, *p)} }
                        }
                    }
                }
            }
        }
    }
}
