//! `CartogramChart`: regions resized by value, as a Dorling cartogram (one circle per
//! region) or a non-contiguous cartogram (each region shrunk about its centroid).
//! Geometry comes from `hydroxy_charts::cartogram`; no map data is bundled.

use dioxus::prelude::*;
use hydroxy_charts::{cartogram, format_value, nav};

pub use hydroxy_charts::cartogram::CartogramKind;
pub use hydroxy_charts::geo::{Projection, Region};

/// Fill and entrance for region `i`: its own chart colour, else the value shade.
fn mark_style(r: &Region, t: Option<f64>, i: usize) -> String {
    let fill = r.colour.map(|c| format!("--fill: var(--chart-{});", c.min(5) + 1)).unwrap_or_default();
    format!("{fill}{}--i: {};", t.map(|t| format!("--t: {t:.3}; ")).unwrap_or_default(), i.min(30))
}

/// Regions resized so their area tracks their value. The default Dorling cartogram draws
/// each region as a labelled circle near where it sits on the map; the non-contiguous
/// kind keeps each region's shape and shrinks it about its centroid, so the densest
/// region keeps its size. A faint outline of the map stays underneath for reference.
/// Hover, touch or focus the map and use the arrow keys to read each region, Home and End
/// to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
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
    let text = current.map(|i| format!("{}: {}", rs[i].name, value_text(rs[i].value))).unwrap_or_default();
    // Biggest first, so small marks stay on top and reachable.
    let mut order: Vec<usize> =
        (0..count).filter(|&i| if dorling { l.cells[i].r > 0.0 } else { l.cells[i].k > 0.0 }).collect();
    order.sort_by(|&a, &b| l.cells[b].r.total_cmp(&l.cells[a].r));
    let tip = current.map(|i| {
        let c = l.cells[i];
        let (x, y) = if dorling { (c.at.0, c.at.1 - c.r) } else { c.centroid };
        let colour =
            rs[i].colour.map_or_else(|| "var(--chart-1)".to_string(), |c| format!("var(--chart-{})", c.min(5) + 1));
        (i, x / l.geo.width * 100.0, y / l.geo.height * 100.0, colour, c.k)
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
                    "aria-label": "{label}. Use the arrow keys to read each region.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
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
                                style: "{mark_style(&rs[i], l.geo.shapes[i].t, i)} --dx: {l.cells[i].centroid.0 - l.cells[i].at.0:.2}px; --dy: {l.cells[i].centroid.1 - l.cells[i].at.1:.2}px;",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
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
                                style: "{mark_style(&rs[i], l.geo.shapes[i].t, i)} transform-origin: {l.cells[i].centroid.0:.2}px {l.cells[i].centroid.1:.2}px; --k: {l.cells[i].k:.4};",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                            }
                        }
                    }
                }
                if let Some((i , left , top , colour , k)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if left > 50.0 { "left" } else { "right" },
                        "data-vertical": if top < 25.0 { "below" } else { "above" },
                        style: "left: {left:.2}%; top: {top:.2}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour};" }
                            "{rs[i].name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "{measure}" }
                                span { "data-part": "tooltip-value", {value_text(rs[i].value)} }
                            }
                            if !dorling && rs[i].value.is_finite() {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "Drawn at" }
                                    span { "data-part": "tooltip-value", "{k * k * 100.0:.0}% of its area" }
                                }
                            }
                        }
                    }
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
