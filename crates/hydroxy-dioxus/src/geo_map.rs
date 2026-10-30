//! `GeoMap`: a choropleth or polygon map over caller-supplied polygons, with optional
//! points on top. Geometry comes from `hydroxy_charts::geo`; no map data is bundled.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, geo, nav};

use crate::callbacks::{select_active, use_hover};
use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Name, row label, value, colour and anchor in viewBox units for one mark.
type Mark = (String, String, f64, String, (f64, f64));

/// Regions shaded on a sequential scale by value, or filled with their own colour when
/// they have one (a polygon map), with optional points on top. Hover, touch or focus the
/// map and use the arrow keys to read regions and then points, Home and End to jump,
/// Escape to clear.
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
    /// Called when the pointer or keyboard moves to a region or point, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a region or point.
    on_select: Option<EventHandler<EventDatum>>,
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
            let colour = colour(r.colour.unwrap_or(0));
            Some((r.name.clone(), measure.clone(), r.value, colour, l.shapes[i].centre))
        } else {
            let p = &pts[i - n];
            Some((p.name.clone(), "Value".to_string(), p.value, colour(p.group), l.points[i - n]?))
        }
    };
    let now = current.map(|i| {
        let (series, index, name, value) =
            if i < n { (0, i, &items[i].name, items[i].value) } else { (1, i - n, &pts[i - n].name, pts[i - n].value) };
        EventDatum { series, index, label: Some(name.clone()), value, node: None }
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let text = current.and_then(mark).map(|(name, _, v, _, _)| format!("{name}: {}", value_text(v)));
    let readout = text.clone().unwrap_or_else(|| "Hover or focus a region".to_string());
    let text = text.unwrap_or_default();
    // `y` is only the `data-vertical` flag (1.0 = below), since the style places the tooltip.
    let tip = current.and_then(mark).map(|(name, row, v, colour, (x, y))| {
        let (fx, fy) = (x / l.width, y / l.height);
        let tip = Tip {
            x: fx,
            y: if fy < 0.25 { 1.0 } else { 0.0 },
            title: name,
            rows: vec![TipRow::new(None, row, value_text(v))],
        };
        (tip, colour, format!("left: {:.2}%; top: {:.2}%;", fx * 100.0, fy * 100.0))
    });
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
                    "aria-label": nav::hinted(&label, nav::HINT_REGIONS),
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onclick: move |_| select_active(Some(datum), on_select),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            select_active(Some(datum), on_select);
                        }
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
                            style: geo::region_style(r, s.t, i, l.shapes.len()),
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
                                style: "fill: {colour(p.group)}; --i: {k}; --ox-n: {l.points.len()};",
                                onpointerenter: move |_| active.set(Some(n + k)),
                                onpointerdown: move |_| active.set(Some(n + k)),
                            }
                        }
                    }
                }
                if let Some((tip, swatch, style)) = tip {
                    ChartTip { tip, swatch, style }
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
