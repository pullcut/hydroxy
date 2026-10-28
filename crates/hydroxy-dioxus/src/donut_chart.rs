//! `DonutChart`: parts of a whole as a ring with a legend. Geometry comes from
//! `hydroxy_charts::donut`. More than six parts fold into "Other".

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{donut, fold_other, format_value, nav};

use crate::callbacks::use_hover;
use crate::plot_kit::{ChartTip, colour_or_muted};

pub use hydroxy_charts::Datum;

/// A ring chart with a total in the middle and a legend with values and shares.
/// Hover or touch a segment or legend row to read it; focus the ring and use the arrow
/// keys to step through segments, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn DonutChart(
    /// The parts, in display order.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Tickets by channel". Used as the accessible name.
    label: String,
    /// Ring diameter in px.
    #[props(default = 180)]
    size: u16,
    /// Ring width in px.
    #[props(default = 22)]
    thickness: u16,
    /// Small caption under the centre figure, such as "Tickets".
    center_label: Option<String>,
    /// Centre figure. Default: the formatted total.
    center_value: Option<String>,
    /// Formats values for the centre, legend, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a segment, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a segment.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let raw = data();
    let folded = raw.len() > 6;
    let parts = fold_other(&raw, 6);
    let n = parts.len();
    // A folded "Other" is the last part and takes the muted colour.
    let slot_colour = move |i: usize| colour_or_muted(i, if folded { n - 1 } else { n });
    let values: Vec<f64> = parts.iter().map(|d| d.value).collect();
    let segs = donut::segments(&values, f64::from(size), f64::from(thickness), 2.0);
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let ring = donut::ring(f64::from(size), f64::from(thickness));
    let centre = center_value.unwrap_or_else(|| fmt(total));
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let now = current.map(|i| EventDatum::category(i, parts[i].label.clone(), parts[i].value));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let key_now = now.clone();
    let pct = |i: usize| format!("{:.0}%", segs.get(i).map_or(0.0, |s| s.fraction) * 100.0);
    let announcement = current.map(|i| format!("{}: {}, {}", parts[i].label, fmt(parts[i].value), pct(i)));
    let tip = current.and_then(|i| donut::anchor(&values, i, f64::from(size), f64::from(thickness)).map(|xy| (i, xy)));

    rsx! {
        figure {
            "data-scope": "donut-chart",
            "data-part": "root",
            "aria-label": "{label}",
            "data-active": current.map(|_| ""),
            style: "--ox-segments: {n};",
            div {
                "data-part": "ring",
                style: "width: {size}px; height: {size}px;",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through segments.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter
                        && let (Some(on_select), Some(d)) = (on_select, key_now.clone())
                    {
                        on_select.call(d);
                    }
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 {size} {size}", width: "{size}", height: "{size}", "aria-hidden": "true",
                    // Keyed by label: a changed share reshapes its own arc; new parts sweep in.
                    for (i, (d, s)) in parts.iter().zip(segs.iter()).enumerate() {
                        path {
                            key: "{d.label}",
                            "data-part": "segment",
                            "data-index": "{i}",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--i: {i};",
                            d: "{s.path}",
                            stroke: slot_colour(i),
                            stroke_width: "{ring.stroke_width}",
                            "pathLength": "1",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: {
                                let picked = EventDatum::category(i, d.label.clone(), d.value);
                                move |_| {
                                    if let Some(on_select) = on_select {
                                        on_select.call(picked.clone());
                                    }
                                }
                            },
                        }
                    }
                }
                div { "data-part": "center", "aria-hidden": "true",
                    span { "data-part": "center-value", "{centre}" }
                    if let Some(l) = center_label {
                        span { "data-part": "center-label", "{l}" }
                    }
                }
                if let Some((i, (x, y))) = tip {
                    ChartTip {
                        tip: Tip {
                            x,
                            y,
                            title: parts[i].label.clone(),
                            rows: vec![
                                TipRow::new(None, "Value", fmt(parts[i].value)),
                                TipRow::new(None, "Share", pct(i)),
                            ],
                        },
                        swatch: slot_colour(i),
                        anchored: true,
                        side: if x > 0.5 { "right" } else { "left" },
                        style: format!("left: {:.3}%; top: {:.3}%;", x * 100.0, y * 100.0),
                    }
                }
            }
            ul { "data-part": "legend", "aria-hidden": "true",
                for (i, d) in parts.iter().enumerate() {
                    li {
                        "data-part": "legend-item",
                        "data-index": "{i}",
                        "data-active": (current == Some(i)).then_some(""),
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerleave: move |_| active.set(None),
                        span { "data-part": "swatch", style: "background: {slot_colour(i)};" }
                        span { "data-part": "legend-label", "{d.label}" }
                        span { "data-part": "legend-value", {fmt(d.value)} }
                        span { "data-part": "legend-share", {pct(i)} }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Part" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Share" }
                    }
                }
                tbody {
                    for (i, d) in parts.iter().enumerate() {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                            td { {pct(i)} }
                        }
                    }
                }
            }
        }
    }
}
