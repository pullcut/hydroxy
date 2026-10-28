//! `FlowMapChart`: curved arrows from origin to destination over base map shapes, as
//! wide as their volume. Geometry comes from `oxide_charts::flow_map`; no map data is bundled.

use dioxus::prelude::*;
use oxide_charts::{flow_map, format_value, nav};

pub use oxide_charts::flow_map::Flow;
pub use oxide_charts::geo::{GeoPoint, Projection, Region};

/// Movements between places, each a curved arrow from its origin to its destination whose
/// width is its volume. Every arrow bows to the same side of its direction, so a flow and
/// its return run side by side. Hover, touch or focus the map and use the arrow keys to
/// step through the flows in the order given, Home and End to jump, Escape to clear; the
/// picked flow and its two ends stand out.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn FlowMapChart(
    /// The places flows run between. Only their name and position are used.
    places: ReadSignal<Vec<GeoPoint>>,
    /// The flows: `Flow::new(from, to, volume)` with indices into `places`.
    flows: ReadSignal<Vec<Flow>>,
    /// Base shapes drawn under the flows, as rings of `(longitude, latitude)`. Their values are ignored.
    #[props(default)]
    regions: ReadSignal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Name of the volume, such as "Passengers". Heads the table column.
    measure: String,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[props(default)]
    projection: Projection,
    /// Width of the biggest flow, in viewBox units (the map is 640 wide).
    #[props(default = 12.0)]
    max_width: f64,
    /// Formats volumes on the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let value_text = move |v: f64| if v.is_finite() { fmt(v) } else { "No data".to_string() };
    let l = use_memo(move || flow_map::layout(&places.read(), &flows.read(), &regions.read(), projection, max_width));
    let l = l.read();
    let ps = places();
    let fs = flows();
    let mut active = use_signal(|| None::<usize>);
    let count = fs.len();
    let current = active().filter(|&i| i < count);
    let name = |i: usize| ps.get(i).map(|p| p.name.clone()).unwrap_or_default();
    let ends = current.map(|i| (fs[i].from, fs[i].to));
    let text = current.map(|i| format!("{} to {}: {}", name(fs[i].from), name(fs[i].to), value_text(fs[i].value))).unwrap_or_default();
    // Widest first in paint order, so the widest flow also draws first.
    let arcs: Vec<(usize, usize, flow_map::Arc)> = l.order.iter().filter_map(|&i| l.arcs[i].clone().map(|a| (i, a))).enumerate().map(|(k, (i, a))| (k, i, a)).collect();
    let dots: Vec<(usize, String, f64, f64)> = ps.iter().zip(&l.geo.points).enumerate().filter_map(|(k, (p, at))| at.map(|(x, y)| (k, p.name.clone(), x, y))).collect();
    let tip = current.and_then(|i| l.arcs[i].as_ref().map(|a| (i, a.mid.0 / l.geo.width * 100.0, a.mid.1 / l.geo.height * 100.0)));
    let (width, view_box) = (l.geo.width, format!("0 0 {} {:.2}", l.geo.width, l.geo.height));

    rsx! {
        figure { "data-scope": "flow-map-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "{view_box}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through the flows.",
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
                    for (k , i , a) in arcs {
                        g {
                            key: "{i}",
                            "data-part": "flow",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--i: {k.min(30)};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            path { "data-part": "hit", d: "{a.path}", stroke_width: "{a.width + 10.0:.2}" }
                            path { "data-part": "line", d: "{a.path}", "pathLength": "1", stroke_width: "{a.width:.2}" }
                            path { "data-part": "head", d: "{a.head}" }
                        }
                    }
                    for (k , n , x , y) in dots {
                        g {
                            key: "{k}",
                            "data-part": "place",
                            "data-end": ends.and_then(|(a, b)| if a == k { Some("origin") } else if b == k { Some("destination") } else { None }),
                            style: "--i: {k.min(30)};",
                            circle { cx: "{x:.2}", cy: "{y:.2}", r: "{flow_map::PLACE_RADIUS:.1}" }
                            text {
                                "data-part": "label",
                                x: if x > width * 0.8 { "{x - 8.0:.2}" } else { "{x + 8.0:.2}" },
                                y: "{y:.2}",
                                text_anchor: if x > width * 0.8 { "end" } else { "start" },
                                "{n}"
                            }
                        }
                    }
                }
                if let Some((i , left , top)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if left > 50.0 { "left" } else { "right" },
                        "data-vertical": if top < 25.0 { "below" } else { "above" },
                        style: "left: {left:.2}%; top: {top:.2}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch" }
                            "{name(fs[i].from)} → {name(fs[i].to)}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "{measure}" }
                                span { "data-part": "tooltip-value", {value_text(fs[i].value)} }
                            }
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{text}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "From" }
                        th { scope: "col", "To" }
                        th { scope: "col", "{measure}" }
                    }
                }
                tbody {
                    for f in fs.iter() {
                        tr {
                            th { scope: "row", {name(f.from)} }
                            td { {name(f.to)} }
                            td { {value_text(f.value)} }
                        }
                    }
                }
            }
        }
    }
}
