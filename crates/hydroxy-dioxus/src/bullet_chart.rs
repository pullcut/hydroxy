//! `BulletChart`: one row per measure, each a bar against a target tick over grey
//! qualitative bands. Each row has its own zero-based scale, from `hydroxy_charts::bullet`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{bullet, format_value, funnel, nav};

use crate::plot_kit::{colour, pct};

pub use hydroxy_charts::bullet::BulletRow;

/// Name of band `j`: the given label, or "Range n".
fn band_name(names: &[String], j: usize) -> String {
    names.get(j).cloned().unwrap_or_else(|| format!("Range {}", j + 1))
}

/// Which band `value` falls in, if any.
fn band_of(edges: &[f64], value: f64) -> Option<usize> {
    edges.iter().position(|&e| value <= e)
}

/// Bullet rows. Hover, touch or focus a row to read it; focus the chart and use the
/// arrow keys to step through rows, Home and End to jump, Escape to clear.
#[component]
pub fn BulletChart(
    /// The measures, top to bottom.
    rows: ReadSignal<Vec<BulletRow>>,
    /// What the chart shows, such as "Quarterly targets". Used as the accessible name.
    label: String,
    /// Names for the bands from the lowest up, such as "Poor", "Fair", "Good".
    #[props(default)]
    range_labels: Vec<String>,
    /// Formats values for the ticks, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let rows_v = rows();
    let n = rows_v.len();
    let layouts: Vec<bullet::RowLayout> = rows_v.iter().map(bullet::layout).collect();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|i| EventDatum::category(i, rows_v[i].label.clone(), rows_v[i].value))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let bands = layouts.iter().map(|l| l.edges.len()).max().unwrap_or(0);
    let has_target = rows_v.iter().any(|r| r.target.is_some());
    let announcement = current
        .map(|i| {
            let r = &rows_v[i];
            let mut text = format!("{}: {}", r.label, fmt(r.value));
            if let Some(t) = r.target {
                text = format!("{text}, target {}", fmt(t));
            }
            if let Some(j) = band_of(&layouts[i].edges, r.value) {
                text = format!("{text}, {}", band_name(&range_labels, j));
            }
            text
        })
        .unwrap_or_default();

    rsx! {
        figure { "data-scope": "bullet-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through measures.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    let key = e.key().to_string();
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                for (i , (r , l)) in rows_v.iter().zip(layouts.iter()).enumerate() {
                    div {
                        key: "{i}",
                        "data-part": "row",
                        "data-index": "{i}",
                        "data-active": (current == Some(i)).then_some(""),
                        style: "--i: {i}; --ox-n: {rows_v.len()};",
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerdown: move |_| active.set(Some(i)),
                        onclick: move |_| {
                            active.set(Some(i));
                            crate::callbacks::select_active(Some(datum), on_select);
                        },
                        div { "data-part": "row-label", "aria-hidden": "true",
                            span { "data-part": "row-name", "{r.label}" }
                            span { "data-part": "row-value", {fmt(r.value)} }
                        }
                        div { "data-part": "track", "aria-hidden": "true",
                            for (j , (a , b)) in l.bands.iter().copied().enumerate() {
                                span { "data-part": "band", "data-band": "{j}", style: "left: {pct(a)}; width: {pct(b - a)};" }
                            }
                            span { "data-part": "measure", style: "width: {pct(l.value)};" }
                            if let Some(t) = l.target {
                                span { "data-part": "target", style: "left: {pct(t)};" }
                            }
                            div { "data-part": "ticks",
                                for t in l.ticks.iter() {
                                    span { "data-part": "tick", style: "left: {pct(t.fraction)};", {fmt(t.value)} }
                                }
                            }
                            if current == Some(i) {
                                div {
                                    "data-part": "tooltip",
                                    "data-side": if l.value > 0.6 { "left" } else { "right" },
                                    style: "left: {pct(l.value)}; top: 50%;",
                                    div { "data-part": "tooltip-label",
                                        span { "data-part": "swatch", style: "background: {colour(0)};" }
                                        "{r.label}"
                                    }
                                    ul {
                                        li { "data-part": "tooltip-row",
                                            span { "data-part": "tooltip-name", "Value" }
                                            span { "data-part": "tooltip-value", {fmt(r.value)} }
                                        }
                                        if let Some(t) = r.target {
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "tooltip-name", "Target" }
                                                span { "data-part": "tooltip-value", {fmt(t)} }
                                            }
                                        }
                                        if let Some(j) = band_of(&l.edges, r.value) {
                                            li { "data-part": "tooltip-row",
                                                span { "data-part": "tooltip-name", "Band" }
                                                span { "data-part": "tooltip-value", {band_name(&range_labels, j)} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ul { "data-part": "legend", "aria-hidden": "true",
                li { "data-part": "legend-item",
                    span { "data-part": "key", "data-key": "measure" }
                    "Value"
                }
                if has_target {
                    li { "data-part": "legend-item",
                        span { "data-part": "key", "data-key": "target" }
                        "Target"
                    }
                }
                for j in 0..bands {
                    li { "data-part": "legend-item",
                        span { "data-part": "key", "data-key": "band", "data-band": "{j}" }
                        {band_name(&range_labels, j)}
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Measure" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Target" }
                        for j in 0..bands {
                            th { scope: "col", {band_name(&range_labels, j)} }
                        }
                    }
                }
                tbody {
                    for (r , l) in rows_v.iter().zip(layouts.iter()) {
                        tr {
                            th { scope: "row", "{r.label}" }
                            td { {fmt(r.value)} }
                            td { {r.target.map(fmt).unwrap_or_default()} }
                            for j in 0..bands {
                                td { {l.edges.get(j).map(|&e| fmt(e)).unwrap_or_default()} }
                            }
                        }
                    }
                }
            }
        }
    }
}
