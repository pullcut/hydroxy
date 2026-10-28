//! `BubbleMapChart`: proportional circles at longitude and latitude over base map
//! shapes. Geometry comes from `hydroxy_charts::geo`; no map data is bundled.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, geo, nav};

use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Circles whose area is their value, drawn over plain regions. Hover, touch or focus the
/// map and use the arrow keys to step through the points in the order given, Home and End
/// to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn BubbleMapChart(
    /// The points. Give each a group (0 to 5) to colour it by series.
    points: ReadSignal<Vec<GeoPoint>>,
    /// Base shapes drawn under the bubbles, as rings of `(longitude, latitude)`. Their values are ignored.
    #[props(default)]
    regions: ReadSignal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Name of the measure, such as "Orders". Heads the table column.
    measure: String,
    /// Series names for the legend, by group index. The legend shows when there are two or more.
    #[props(default)]
    groups: ReadSignal<Vec<String>>,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[props(default)]
    projection: Projection,
    /// Radius of the largest bubble, in viewBox units (the map is 640 wide).
    #[props(default = 28.0)]
    max_radius: f64,
    /// Formats values on the tooltip, legend and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let value_text = move |v: f64| if v.is_finite() { fmt(v) } else { "No data".to_string() };
    let mut active = use_signal(|| None::<usize>);
    let pts = points();
    let names = groups();
    let l = geo::layout(&regions(), &pts, projection);
    let max = pts.iter().map(|p| p.value).filter(|v| v.is_finite()).fold(0.0, f64::max);
    let current = active().filter(|&i| i < pts.len());
    let cur = if on_hover.is_some() || on_select.is_some() {
        current
            .map(|i| EventDatum { series: pts[i].group, ..EventDatum::category(i, pts[i].name.clone(), pts[i].value) })
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let text = current.map(|i| format!("{}: {}", pts[i].name, value_text(pts[i].value))).unwrap_or_default();
    // `x` picks the side; `y` is only the `data-vertical` flag (1.0 = below), since the style places it.
    let tip = current.and_then(|i| {
        let (x, y) = l.points[i]?;
        let p = &pts[i];
        let (left, top) = (x / l.width * 100.0, y / l.height * 100.0);
        let mut rows = vec![TipRow::new(None, measure.clone(), value_text(p.value))];
        if let Some(g) = names.get(p.group) {
            rows.push(TipRow::new(None, "Series", g.clone()));
        }
        let tip = Tip { x: left / 100.0, y: if top < 25.0 { 1.0 } else { 0.0 }, title: p.name.clone(), rows };
        Some((tip, colour(p.group), format!("left: {left:.2}%; top: {top:.2}%;")))
    });
    let count = pts.len();
    let bubbles: Vec<(usize, f64, f64, f64, usize)> = geo::draw_order(&pts)
        .into_iter()
        .filter_map(|i| {
            l.points[i]
                .map(|(x, y)| (i, x, y, geo::bubble_radius(pts[i].value, max, max_radius).max(2.0), pts[i].group))
        })
        .collect();
    let view_box = format!("0 0 {} {:.2}", l.width, l.height);

    rsx! {
        figure { "data-scope": "bubble-map-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "{view_box}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{nav::hinted(&label, nav::HINT_POINTS)}",
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
                        for s in l.shapes.iter() {
                            path { d: "{s.path}" }
                        }
                    }
                    for (i , x , y , r , g) in bubbles {
                        circle {
                            key: "{i}",
                            "data-part": "bubble",
                            "data-active": (current == Some(i)).then_some(""),
                            cx: "{x:.2}",
                            cy: "{y:.2}",
                            r: "{r:.2}",
                            style: "fill: {colour(g)}; --i: {i.min(40)};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: move |_| {
                                active.set(Some(i));
                                crate::callbacks::select_active(Some(datum), on_select);
                            },
                        }
                    }
                }
                if let Some((tip, swatch, style)) = tip {
                    ChartTip { tip, swatch, style }
                }
            }
            if names.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , g) in names.iter().take(6).enumerate() {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{g}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{text}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Name" }
                        th { scope: "col", "Longitude" }
                        th { scope: "col", "Latitude" }
                        th { scope: "col", "{measure}" }
                    }
                }
                tbody {
                    for p in pts.iter() {
                        tr {
                            th { scope: "row", "{p.name}" }
                            td { "{p.lon:.2}" }
                            td { "{p.lat:.2}" }
                            td { if p.value.is_finite() { {fmt(p.value)} } else { "–" } }
                        }
                    }
                }
            }
        }
    }
}
