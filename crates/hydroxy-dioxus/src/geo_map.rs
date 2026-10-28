//! `GeoMap`: a choropleth or polygon map over caller-supplied polygons, with optional
//! points on top. Geometry comes from `hydroxy_charts::geo`; no map data is bundled.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, geo, nav};

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Region fill: its own chart colour on a polygon map, else the value shade.
fn region_style(r: &Region, t: Option<f64>, i: usize) -> String {
    let fill = r.colour.map(|c| format!("fill: var(--chart-{});", c + 1)).unwrap_or_default();
    format!("{fill}{}--i: {};", t.map(|t| format!("--t: {t:.3}; ")).unwrap_or_default(), i.min(40))
}

/// Name, row label, value, colour and anchor in viewBox units for one mark.
type Mark = (String, String, f64, String, (f64, f64));

/// Regions shaded on a sequential scale by value, or filled with their own colour when
/// they have one (a polygon map), with optional points on top. Hover, touch or focus the
/// map and use the arrow keys to read regions and then points, Home and End to jump,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn GeoMap(
    /// The regions, as rings of `(longitude, latitude)` degrees.
    regions: ReadSignal<Vec<Region>>,
    /// Points drawn over the regions, such as offices or stores.
    #[props(default)]
    points: ReadSignal<Vec<GeoPoint>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Name of the measure, such as "Customers per 1k people". Heads the table column.
    measure: String,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[props(default)]
    projection: Projection,
    /// Formats values on the legend, tooltip, readout and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let value_text = move |v: f64| if v.is_finite() { fmt(v) } else { "No data".to_string() };
    let mut active = use_signal(|| None::<usize>);
    let items = regions();
    let pts = points();
    let l = geo::layout(&items, &pts, projection);
    let n = items.len();
    let total = n + pts.len();
    let current = active().filter(|&i| i < total);
    // (name, row label, value, colour, anchor) for mark `i`.
    let mark = |i: usize| -> Option<Mark> {
        if i < n {
            let r = &items[i];
            let colour = r.colour.map_or_else(|| "var(--chart-1)".to_string(), |c| format!("var(--chart-{})", c + 1));
            Some((r.name.clone(), measure.clone(), r.value, colour, l.shapes[i].centre))
        } else {
            let p = &pts[i - n];
            Some((
                p.name.clone(),
                "Value".to_string(),
                p.value,
                format!("var(--chart-{})", p.group.min(5) + 1),
                l.points[i - n]?,
            ))
        }
    };
    let text = current.and_then(mark).map(|(name, _, v, _, _)| format!("{name}: {}", value_text(v)));
    let readout = text.clone().unwrap_or_else(|| "Hover or focus a region".to_string());
    let text = text.unwrap_or_default();
    let tip = current
        .and_then(mark)
        .map(|(name, row, v, colour, (x, y))| (name, row, v, colour, x / l.width * 100.0, y / l.height * 100.0));
    let shaded = items.iter().any(|r| r.colour.is_none() && r.value.is_finite());
    let view_box = format!("0 0 {} {:.2}", l.width, l.height);

    rsx! {
        figure { "data-scope": "geo-map", "data-part": "root", "data-active": current.map(|_| ""),
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
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), total) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for (i , (r , s)) in items.iter().zip(l.shapes.iter()).enumerate() {
                        path {
                            key: "{r.id}",
                            "data-part": "region",
                            "data-empty": (s.t.is_none() && r.colour.is_none()).then_some(""),
                            "data-active": (current == Some(i)).then_some(""),
                            d: "{s.path}",
                            style: region_style(r, s.t, i),
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                        }
                    }
                    for (k , (p , at)) in pts.iter().zip(l.points.iter()).enumerate() {
                        if let Some((x , y)) = *at {
                            circle {
                                key: "p{k}",
                                "data-part": "point",
                                "data-active": (current == Some(n + k)).then_some(""),
                                cx: "{x:.2}",
                                cy: "{y:.2}",
                                r: "5",
                                style: "fill: var(--chart-{p.group.min(5) + 1}); --i: {k.min(40)};",
                                onpointerenter: move |_| active.set(Some(n + k)),
                                onpointerdown: move |_| active.set(Some(n + k)),
                            }
                        }
                    }
                }
                if let Some((name , row , v , colour , left , top)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if left > 50.0 { "left" } else { "right" },
                        "data-vertical": if top < 25.0 { "below" } else { "above" },
                        style: "left: {left:.2}%; top: {top:.2}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour};" }
                            "{name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "{row}" }
                                span { "data-part": "tooltip-value", {value_text(v)} }
                            }
                        }
                    }
                }
            }
            div { "data-part": "legend", "aria-hidden": "true",
                output { "data-part": "readout", "{readout}" }
                if shaded {
                    div { "data-part": "scale",
                        span { {fmt(l.range.0)} }
                        span { "data-part": "ramp" }
                        span { {fmt(l.range.1)} }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{text}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Name" }
                        th { scope: "col", "Kind" }
                        th { scope: "col", "{measure}" }
                    }
                }
                tbody {
                    for r in items.iter() {
                        tr {
                            th { scope: "row", "{r.name}" }
                            td { "Region" }
                            td { {value_text(r.value)} }
                        }
                    }
                    for p in pts.iter() {
                        tr {
                            th { scope: "row", "{p.name}" }
                            td { "Point" }
                            td { {value_text(p.value)} }
                        }
                    }
                }
            }
        }
    }
}
