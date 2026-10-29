//! `NestedDonutChart`: concentric rings, innermost first. A ring is either one level of
//! a tree, with children under their parent's sweep, or a separate series over the same
//! categories. Geometry comes from `hydroxy_charts::nested_donut`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{chord, format_value, nav, nested_donut};

use crate::plot_kit::colour_or_muted;
use crate::sets_kit::SetsTip;

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::hierarchy::Tree;
pub use hydroxy_charts::nested_donut::{Part, Ring, levels, series};

/// A nested donut. Build the rings with [`levels`] (one ring per tree level) or [`series`]
/// (one ring per series). The whole chart sweeps in clockwise as one motion. Hover or
/// touch a segment to read it; focus the chart and use the arrow keys to step through the
/// segments ring by ring, Home and End to jump, Escape to clear.
#[component]
pub fn NestedDonutChart(
    /// The rings, innermost first.
    rings: ReadSignal<Vec<Ring>>,
    /// What the chart shows, such as "Traffic by source, 2024 and 2025". Used as the accessible name.
    label: String,
    /// Diameter in px.
    #[props(default = 280)]
    size: u16,
    /// The empty centre's share of the radius, 0 to 0.9.
    #[props(default = 0.42)]
    hole: f64,
    /// Small caption under the centre figure. Default: the outer ring's name.
    center_label: Option<String>,
    /// Formats values for the centre, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let r = rings();
    let lay = nested_donut::layout(&r, f64::from(size), hole, 3.0, 2.0);
    let mut active = use_signal(|| None::<usize>);
    let marks = lay.segments.len();
    let current = active().filter(|&i| i < marks);
    let now = current.map(|i| {
        let seg = &lay.segments[i];
        let part = &r[seg.ring].parts[seg.part];
        EventDatum { series: seg.ring, index: seg.part, label: Some(part.label.clone()), value: part.value, node: None }
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let tip = current.and_then(|i| lay.tip(&r, i, &fmt));
    let announcement = tip.as_ref().map(hydroxy_charts::plot::Tip::announce).unwrap_or_default();
    let (centre_value, centre_name) = if let Some(seg) = current.map(|i| &lay.segments[i]) {
        let part = &r[seg.ring].parts[seg.part];
        (fmt(part.value), part.label.clone())
    } else {
        let outer = lay.rings.last().map_or(0.0, |x| x.total);
        (fmt(outer), center_label.unwrap_or_else(|| r.last().map(|x| x.name.clone()).unwrap_or_default()))
    };
    let names = nested_donut::legend(&r);
    let slots = names.len();
    let legend = names
        .into_iter()
        .enumerate()
        .filter(|(i, _)| chord::slot(*i, slots).is_some())
        .map(|(i, t)| (chord::colour(i, slots), t))
        .chain((slots > 6).then(|| (colour_or_muted(slots, slots), "Other".to_string())));
    let ring_names: Vec<String> = r.iter().map(|x| x.name.clone()).collect();
    let centre_width = lay.hole * 2.0 - 8.0;

    rsx! {
        figure { "data-scope": "nested-donut-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div {
                "data-part": "plot",
                style: "width: {size}px; height: {size}px;",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the segments.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| crate::callbacks::select_active(Some(datum), on_select),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), marks) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg {
                    view_box: "0 0 {size} {size}",
                    width: "{size}",
                    height: "{size}",
                    "aria-hidden": "true",
                    for (k , ring) in lay.rings.iter().enumerate() {
                        g {
                            key: "{k}",
                            "data-part": "ring",
                            "data-ring": "{k}",
                            transform: "translate({ring.offset:.3} {ring.offset:.3})",
                            for (i , seg) in lay.segments.iter().enumerate().filter(|(_, s)| s.ring == k) {
                                path {
                                    key: "{i}",
                                    "data-part": "segment",
                                    "data-active": (current == Some(i)).then_some(""),
                                    d: "{seg.path}",
                                    stroke: lay.colour(&r, i),
                                    stroke_width: "{ring.thickness}",
                                    stroke_opacity: "{r[k].opacity}",
                                    onpointerenter: move |_| active.set(Some(i)),
                                    onpointerdown: move |_| active.set(Some(i)),
                                }
                            }
                        }
                    }
                }
                div { "data-part": "center", "aria-hidden": "true", style: "width: {centre_width:.1}px;",
                    span { "data-part": "center-value", "{centre_value}" }
                    span { "data-part": "center-label", "{centre_name}" }
                }
                if let Some(tip) = tip {
                    SetsTip { tip }
                }
            }
            div { "data-part": "key", "aria-hidden": "true",
                ul { "data-part": "legend",
                    for (c , t) in legend {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {c};" }
                            "{t}"
                        }
                    }
                }
                if ring_names.len() > 1 {
                    p { "data-part": "ring-key", "Rings from the centre out: {ring_names.join(\", \")}" }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Ring" }
                        th { scope: "col", "Part" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Share" }
                    }
                }
                tbody {
                    for seg in lay.segments.iter() {
                        tr {
                            td { "{r[seg.ring].name}" }
                            th { scope: "row", "{r[seg.ring].parts[seg.part].label}" }
                            td { {fmt(r[seg.ring].parts[seg.part].value)} }
                            td { {hydroxy_charts::upset::percent(seg.fraction, 1.0)} }
                        }
                    }
                }
            }
        }
    }
}
