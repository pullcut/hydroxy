//! `FunnelChart`: stages that narrow to the next, with the conversion between each
//! pair. Geometry comes from `hydroxy_charts::funnel`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{format_value, funnel, nav};

pub use hydroxy_charts::{Datum, Orientation};

use crate::callbacks::{select_active, use_hover};
use crate::plot_kit::{colour, pct};

fn percent(f: f64) -> String {
    format!("{}%", format_value(f * 100.0))
}

/// A funnel, top to bottom or left to right. Hover, touch or focus a stage to read it;
/// focus the chart and use the arrow keys to step through stages, Home and End to
/// jump, Escape to clear.
#[component]
pub fn FunnelChart(
    /// The stages, in order.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Signup funnel". Used as the accessible name.
    label: String,
    /// Which way the funnel runs. Default top to bottom.
    #[props(default)]
    orientation: Orientation,
    /// Plot height in px.
    #[props(default = 280)]
    height: u16,
    /// Formats values for the labels, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a stage, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a stage.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let rows = data();
    let n = rows.len();
    let stages = funnel::layout(&rows.iter().map(|x| x.value).collect::<Vec<_>>(), 0.03);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let now = current.map(|i| EventDatum::category(i, rows[i].label.clone(), rows[i].value));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let arrow = if orientation == Orientation::Vertical { "\u{2193}" } else { "\u{2192}" };
    let announcement = current
        .map(|i| {
            let text = format!("{}: {}", rows[i].label, fmt(rows[i].value));
            match stages[i].conversion {
                Some(c) => format!("{text}, {} of the previous stage", percent(c)),
                None => text,
            }
        })
        .unwrap_or_default();
    let tip = current.map(|i| {
        let s = stages[i];
        let mid = pct(f64::midpoint(s.start, s.end));
        let pos = match orientation {
            Orientation::Vertical => format!("left: 50%; top: {mid};"),
            Orientation::Horizontal => format!("left: {mid}; top: 50%;"),
        };
        let side = if orientation == Orientation::Horizontal && s.start + s.end > 1.0 { "left" } else { "right" };
        (i, s, pos, side)
    });

    rsx! {
        figure { "data-scope": "funnel-chart", "data-part": "root", "data-orientation": orientation.attr(), style: "--ox-segments: {n};",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through stages.",
                style: "height: {height}px;",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| select_active(Some(datum), on_select),
                onkeydown: move |e| {
                    let key = e.key().to_string();
                    if key == "Enter" {
                        select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                div { "data-part": "labels", "aria-hidden": "true",
                    for (i , (d , s)) in rows.iter().zip(stages.iter()).enumerate() {
                        div {
                            "data-part": "stage-label",
                            "data-index": "{i}",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--at: {pct(f64::midpoint(s.start, s.end))}; --i: {i};",
                            onpointerenter: move |_| active.set(Some(i)),
                            span { "data-part": "stage-name", "{d.label}" }
                            span { "data-part": "stage-value", {fmt(d.value)} }
                        }
                    }
                }
                div { "data-part": "area", style: "--ox-n: {stages.len()};",
                    svg { view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                        for (i , s) in stages.iter().enumerate() {
                            polygon {
                                key: "{i}",
                                "data-part": "stage",
                                "data-index": "{i}",
                                "data-active": (current == Some(i)).then_some(""),
                                style: "--i: {i};",
                                points: "{funnel::polygon(s, orientation)}",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                            }
                        }
                    }
                    for (i , s , c) in stages.iter().enumerate().filter_map(|(i, s)| s.conversion.map(|c| (i, s, c))) {
                        span { "data-part": "conversion", "aria-hidden": "true", style: "--at: {pct(s.start)}; --i: {i};",
                            "{arrow} {percent(c)}"
                        }
                    }
                    if let Some((i , s , pos , side)) = tip {
                        div { "data-part": "tooltip", "data-side": side, style: "{pos}", "aria-hidden": "true",
                            div { "data-part": "tooltip-label",
                                span { "data-part": "swatch", style: "background: {colour(0)};" }
                                "{rows[i].label}"
                            }
                            ul {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "Value" }
                                    span { "data-part": "tooltip-value", {fmt(rows[i].value)} }
                                }
                                if let Some(c) = s.conversion {
                                    li { "data-part": "tooltip-row",
                                        span { "data-part": "tooltip-name", "From previous" }
                                        span { "data-part": "tooltip-value", {percent(c)} }
                                    }
                                }
                                if let Some(f) = s.of_first.filter(|_| i > 0) {
                                    li { "data-part": "tooltip-row",
                                        span { "data-part": "tooltip-name", "From first" }
                                        span { "data-part": "tooltip-value", {percent(f)} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Stage" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "From previous" }
                        th { scope: "col", "From first" }
                    }
                }
                tbody {
                    for (d , s) in rows.iter().zip(stages.iter()) {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                            td { {s.conversion.map(percent).unwrap_or_default()} }
                            td { {s.of_first.map(percent).unwrap_or_default()} }
                        }
                    }
                }
            }
        }
    }
}
