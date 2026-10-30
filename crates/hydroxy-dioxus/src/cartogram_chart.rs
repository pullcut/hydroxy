//! `CartogramChart`: regions resized by value, as a Dorling cartogram (one circle per
//! region) or a non-contiguous cartogram (each region shrunk about its centroid).
//! Geometry comes from `hydroxy_charts::cartogram`; no map data is bundled.

use std::fmt::Write as _;

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{cartogram, format_value, nav};

use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::cartogram::CartogramKind;
pub use hydroxy_charts::geo::{Projection, Region};

/// Fill and entrance for region `i` of `n`: its own chart colour, else the value shade.
fn mark_style(r: &Region, t: Option<f64>, i: usize, n: usize) -> String {
    let mut style = String::new();
    if let Some(fill) = r.colour {
        let _ = write!(style, "--fill: {};", colour(fill));
    }
    if let Some(t) = t {
        let _ = write!(style, "--t: {t:.3}; ");
    }
    let _ = write!(style, "--i: {i}; --ox-n: {n};");
    style
}

/// Regions resized so their area tracks their value. The default Dorling cartogram draws
/// each region as a labelled circle near where it sits on the map; the non-contiguous
/// kind keeps each region's shape and shrinks it about its centroid, so the densest
/// region keeps its size. A faint outline of the map stays underneath for reference.
/// Hover, touch or focus the map and use the arrow keys to read each region, Home and End
/// to jump, Escape to clear.
#[component]
pub fn CartogramChart(
    /// The regions, as rings of `(longitude, latitude)` degrees. Dorling circles are
    /// labelled with each region's id, so keep ids short (state or country codes).
    regions: ReadSignal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Name of the measure, such as "Population (m)". Heads the table column.
    measure: String,
    /// Dorling circles (default) or non-contiguous shapes.
    #[props(default)]
    kind: CartogramKind,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[props(default)]
    projection: Projection,
    /// Formats values on the legend, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let value_text = move |v: f64| if v.is_finite() { fmt(v) } else { "No data".to_string() };
    let rs = regions();
    let l = use_memo(move || cartogram::layout(&regions.read(), projection));
    let l = l.read();
    let mut active = use_signal(|| None::<usize>);
    let dorling = kind == CartogramKind::Dorling;
    let count = rs.len().min(l.cells.len());
    let current = active().filter(|&i| i < count);
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|i| EventDatum::category(i, rs[i].name.clone(), rs[i].value))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let text = current.map(|i| format!("{}: {}", rs[i].name, value_text(rs[i].value))).unwrap_or_default();
    // Biggest first, so small marks stay on top and reachable.
    let mut order: Vec<usize> =
        (0..count).filter(|&i| if dorling { l.cells[i].r > 0.0 } else { l.cells[i].k > 0.0 }).collect();
    order.sort_by(|&a, &b| l.cells[b].r.total_cmp(&l.cells[a].r));
    let tip = current.map(|i| {
        let c = l.cells[i];
        let (x, y) = if dorling { (c.at.0, c.at.1 - c.r) } else { c.centroid };
        let swatch = rs[i].colour.map_or_else(|| colour(0), colour);
        let (left, top) = (x / l.geo.width * 100.0, y / l.geo.height * 100.0);
        let mut rows = vec![TipRow::new(None, measure.clone(), value_text(rs[i].value))];
        if !dorling && rs[i].value.is_finite() {
            rows.push(TipRow::new(None, "Drawn at", format!("{:.0}% of its area", c.k * c.k * 100.0)));
        }
        // `x` picks the side; `y` is only the `data-vertical` flag (1.0 = below), since the style places it.
        let tip = Tip { x: left / 100.0, y: if top < 25.0 { 1.0 } else { 0.0 }, title: rs[i].name.clone(), rows };
        (tip, swatch, format!("left: {left:.2}%; top: {top:.2}%;"))
    });
    let shaded = rs.iter().any(|r| r.colour.is_none() && r.value.is_finite());
    let view_box = format!("0 0 {} {:.2}", l.geo.width, l.geo.height);

    rsx! {
        figure {
            "data-scope": "cartogram-chart",
            "data-part": "root",
            "data-kind": if dorling { "dorling" } else { "non-contiguous" },
            "data-active": current.map(|_| ""),
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "{view_box}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{nav::hinted(&label, nav::HINT_REGIONS)}",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), count) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    g { "data-part": "land", "aria-hidden": "true",
                        for s in l.geo.shapes.iter() {
                            path { d: "{s.path}" }
                        }
                    }
                    for i in order {
                        if dorling {
                            g {
                                key: "{i}",
                                "data-part": "cell",
                                "data-active": (current == Some(i)).then_some(""),
                                style: "{mark_style(&rs[i], l.geo.shapes[i].t, i, rs.len().min(l.cells.len()))} --dx: {l.cells[i].centroid.0 - l.cells[i].at.0:.2}px; --dy: {l.cells[i].centroid.1 - l.cells[i].at.1:.2}px;",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                onclick: move |_| {
                                    active.set(Some(i));
                                    crate::callbacks::select_active(Some(datum), on_select);
                                },
                                circle { "data-part": "circle", cx: "{l.cells[i].at.0:.2}", cy: "{l.cells[i].at.1:.2}", r: "{l.cells[i].r:.2}" }
                                if l.cells[i].r >= 11.0 {
                                    text { "data-part": "label", x: "{l.cells[i].at.0:.2}", y: "{l.cells[i].at.1:.2}", "{rs[i].id}" }
                                }
                            }
                        } else {
                            path {
                                key: "{i}",
                                "data-part": "region",
                                "data-active": (current == Some(i)).then_some(""),
                                d: "{l.geo.shapes[i].path}",
                                style: "{mark_style(&rs[i], l.geo.shapes[i].t, i, rs.len().min(l.cells.len()))} transform-origin: {l.cells[i].centroid.0:.2}px {l.cells[i].centroid.1:.2}px; --k: {l.cells[i].k:.4};",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                onclick: move |_| {
                                    active.set(Some(i));
                                    crate::callbacks::select_active(Some(datum), on_select);
                                },
                            }
                        }
                    }
                }
                if let Some((tip, swatch, style)) = tip {
                    ChartTip { tip, swatch, style }
                }
            }
            div { "data-part": "legend", "aria-hidden": "true",
                output { "data-part": "readout",
                    if current.is_some() { "{text}" } else { "Hover or focus a region" }
                }
                if shaded {
                    div { "data-part": "scale",
                        span { {fmt(l.geo.range.0)} }
                        span { "data-part": "ramp" }
                        span { {fmt(l.geo.range.1)} }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{text}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Region" }
                        th { scope: "col", "{measure}" }
                    }
                }
                tbody {
                    for r in rs.iter() {
                        tr {
                            th { scope: "row", "{r.name}" }
                            td { {value_text(r.value)} }
                        }
                    }
                }
            }
        }
    }
}
