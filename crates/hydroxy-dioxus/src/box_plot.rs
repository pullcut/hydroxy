//! `BoxPlot`: one box per group, with quartiles, whiskers at 1.5 IQR and outliers.
//! Statistics and heights come from `hydroxy_charts::boxplot`; layout is plain CSS.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{boxplot, format_value, nav};

use crate::plot_kit::pct;
pub use hydroxy_charts::boxplot::Group;

/// Where the median sits down an element spanning `bottom..top`, as a CSS length from its top,
/// so the box opens out from the median.
fn mid(top: f64, bottom: f64, at: f64) -> String {
    let span = top - bottom;
    format!("{:.2}%", if span > 0.0 { ((top - at) / span * 100.0).clamp(0.0, 100.0) } else { 50.0 })
}

/// A box plot of several groups on one shared value axis. Hover or touch a box to read
/// its numbers; focus the plot and use the arrow keys to step through groups, Home and
/// End to jump, Escape to clear.
#[component]
pub fn BoxPlot(
    /// The groups, left to right, each with its raw values.
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Deploy time by team". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the group labels.
    #[props(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let groups_v = groups();
    let layout = boxplot::layout(&groups_v.iter().map(|g| g.values.clone()).collect::<Vec<_>>(), 5);
    let n = groups_v.len();
    let summary = |name: &str, geom: Option<&boxplot::BoxGeom>| match geom {
        Some(b) => {
            let s = &b.stats;
            format!(
                "{name}: median {}, quartiles {} to {}, whiskers {} to {}, {} outliers, {} values",
                fmt(s.median),
                fmt(s.q1),
                fmt(s.q3),
                fmt(s.low),
                fmt(s.high),
                s.outliers.len(),
                s.n,
            )
        }
        None => format!("{name}: no data"),
    };
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|i| {
            let median = layout.boxes.get(i).and_then(Option::as_ref).map_or(f64::NAN, |b| b.stats.median);
            EventDatum::category(i, groups_v[i].label.clone(), median)
        })
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let announcement = current.map(|i| summary(&groups_v[i].label, layout.boxes.get(i).and_then(Option::as_ref)));

    rsx! {
        figure { "data-scope": "box-plot", "data-part": "root",
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.value == 0.0).then_some(""),
                            style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {fmt(g.value)} }
                        }
                    }
                }
                div {
                    "data-part": "columns",
                    role: "group",
                    tabindex: "0",
                    "aria-label": "{nav::hinted(&label, nav::HINT_GROUPS)}",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for (i , (g , geom)) in groups_v.iter().zip(layout.boxes.iter()).enumerate() {
                        div {
                            key: "{i}",
                            "data-part": "column",
                            role: "img",
                            "aria-label": summary(&g.label, geom.as_ref()),
                            "data-active": (current == Some(i)).then_some(""),
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: move |_| {
                                active.set(Some(i));
                                crate::callbacks::select_active(Some(datum), on_select);
                            },
                            style: "--i: {i};",
                            if let Some(b) = geom {
                                div { "data-part": "whisker", style: "bottom: {pct(b.low)}; height: {pct(b.high - b.low)}; --ox-mid: {mid(b.high, b.low, b.median)};" }
                                div { "data-part": "cap", style: "bottom: {pct(b.low)};" }
                                div { "data-part": "cap", style: "bottom: {pct(b.high)};" }
                                div { "data-part": "box", style: "bottom: {pct(b.q1)}; height: {pct(b.q3 - b.q1)}; --ox-mid: {mid(b.q3, b.q1, b.median)};" }
                                div { "data-part": "median", style: "bottom: {pct(b.median)};" }
                                for (oi , o) in b.outliers.iter().enumerate() {
                                    span { key: "{oi}", "data-part": "outlier", style: "bottom: {pct(*o)};" }
                                }
                                div {
                                    "data-part": "tooltip",
                                    "data-side": if i * 3 < n { "start" } else if i * 3 >= n * 2 { "end" } else { "" },
                                    "aria-hidden": "true",
                                    div { "data-part": "tooltip-label", "{g.label}" }
                                    dl {
                                        dt { "Upper whisker" }
                                        dd { {fmt(b.stats.high)} }
                                        dt { "Q3" }
                                        dd { {fmt(b.stats.q3)} }
                                        dt { "Median" }
                                        dd { {fmt(b.stats.median)} }
                                        dt { "Q1" }
                                        dd { {fmt(b.stats.q1)} }
                                        dt { "Lower whisker" }
                                        dd { {fmt(b.stats.low)} }
                                        dt { "Outliers" }
                                        dd { "{b.stats.outliers.len()}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "labels", "aria-hidden": "true",
                for (i , g) in groups_v.iter().enumerate() {
                    span {
                        key: "{i}",
                        "data-part": "label",
                        "data-slot": "",
                        style: "left: {pct((i as f64 + 0.5) / n.max(1) as f64)};",
                        "{g.label}"
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Group" }
                        th { scope: "col", "Count" }
                        th { scope: "col", "Lower whisker" }
                        th { scope: "col", "Q1" }
                        th { scope: "col", "Median" }
                        th { scope: "col", "Q3" }
                        th { scope: "col", "Upper whisker" }
                        th { scope: "col", "Outliers" }
                    }
                }
                tbody {
                    for (g , b) in groups_v.iter().zip(layout.boxes.iter()) {
                        tr {
                            th { scope: "row", "{g.label}" }
                            if let Some(b) = b {
                                td { "{b.stats.n}" }
                                td { {fmt(b.stats.low)} }
                                td { {fmt(b.stats.q1)} }
                                td { {fmt(b.stats.median)} }
                                td { {fmt(b.stats.q3)} }
                                td { {fmt(b.stats.high)} }
                                td { {b.stats.outliers.iter().map(|&o| fmt(o)).collect::<Vec<_>>().join(", ")} }
                            } else {
                                td { colspan: "7", "No data" }
                            }
                        }
                    }
                }
            }
        }
    }
}
