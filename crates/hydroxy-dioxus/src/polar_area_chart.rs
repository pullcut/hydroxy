//! `PolarAreaChart`: a Nightingale rose. Every category gets the same angle and its
//! radius encodes the value. Geometry comes from `hydroxy_charts::polar`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, funnel, nav, polar};

use crate::plot_kit::{ChartTip, colour, pct};

pub use hydroxy_charts::Datum;

/// Which side of its anchor a label around the rose sits on.
fn side(x: f64) -> &'static str {
    if x < 0.45 {
        "left"
    } else if x > 0.55 {
        "right"
    } else {
        "center"
    }
}

/// A rose of equal-angle wedges, clockwise from 12 o'clock. Hover, touch or focus a
/// wedge to read it; focus the chart and use the arrow keys to step through wedges,
/// Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn PolarAreaChart(
    /// The categories, clockwise from 12 o'clock. Months and hours read well.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Incidents by month". Used as the accessible name.
    label: String,
    /// Rose diameter in px.
    #[props(default = 280)]
    size: u16,
    /// Formats values for the rings, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let rows = data();
    let n = rows.len();
    let layout = polar::layout(&rows.iter().map(|x| x.value).collect::<Vec<_>>(), 0.03);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let now = current.map(|i| EventDatum::category(i, rows[i].label.clone(), rows[i].value));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let announcement = current.map(|i| format!("{}: {}", rows[i].label, fmt(rows[i].value))).unwrap_or_default();
    let tip = current.map(|i| (i, polar::at(layout.wedges[i].radius.max(0.2) * 0.66, layout.wedges[i].mid)));

    rsx! {
        figure { "data-scope": "polar-area-chart", "data-part": "root", style: "--ox-segments: {n};",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through categories.",
                style: "width: {size}px;",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| crate::callbacks::select_active(Some(datum), on_select),
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
                svg { view_box: "0 0 100 100", "aria-hidden": "true",
                    for r in layout.rings.iter() {
                        circle { "data-part": "ring", cx: "50", cy: "50", r: "{r.radius * polar::R:.3}" }
                    }
                    for (i , w) in layout.wedges.iter().enumerate() {
                        path {
                            key: "{i}",
                            "data-part": "wedge",
                            "data-index": "{i}",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--i: {i};",
                            d: "{w.path}",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                        }
                    }
                }
                for r in layout.rings.iter() {
                    span { "data-part": "ring-label", "aria-hidden": "true", style: "top: {pct(0.5 - r.radius / 2.0)};", {fmt(r.value)} }
                }
                for (i , (d , w)) in rows.iter().zip(layout.wedges.iter()).enumerate() {
                    {
                        let (x, y) = polar::at(1.06, w.mid);
                        rsx! {
                            span {
                                "data-part": "category",
                                "data-index": "{i}",
                                "data-side": side(x),
                                "aria-hidden": "true",
                                style: "left: {pct(x)}; top: {pct(y)};",
                                "{d.label}"
                            }
                        }
                    }
                }
                if let Some((i , (x , y))) = tip {
                    ChartTip {
                        tip: Tip { x, y: 1.0 - y, title: rows[i].label.clone(), rows: vec![TipRow::new(None, "Value", fmt(rows[i].value))] },
                        swatch: colour(0),
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Category" }
                        th { scope: "col", "Value" }
                    }
                }
                tbody {
                    for d in rows.iter() {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                        }
                    }
                }
            }
        }
    }
}
