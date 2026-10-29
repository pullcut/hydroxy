//! `HexbinMapChart`: points binned into hexagons over projected map shapes, each
//! hexagon shaded by how many points it holds. Geometry comes from `hydroxy_charts::geo`
//! and `hydroxy_charts::hexbin`; no map data is bundled.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, geo, hexbin, nav};

use crate::callbacks::{select_active, use_hover};
use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Hexagons over a map, darker where more points fall. Hover, touch or focus the map and
/// use the arrow keys to step through the hexagons from the top left, Home and End to
/// jump, Escape to clear.
#[component]
pub fn HexbinMapChart(
    /// The points to bin. Their values add up into each hexagon's total.
    points: ReadSignal<Vec<GeoPoint>>,
    /// Base shapes drawn under the hexagons, as rings of `(longitude, latitude)`. Their values are ignored.
    #[props(default)]
    regions: ReadSignal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Hexagon radius, centre to corner, in viewBox units (the map is 640 wide).
    #[props(default = 16.0)]
    radius: f64,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[props(default)]
    projection: Projection,
    /// Formats each hexagon's total value. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a hexagon, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a hexagon.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let pts = points();
    let l = geo::layout(&regions(), &pts, projection);
    let bins = hexbin::cells(&pts, &l.points, radius);
    let count = bins.len();
    let most = bins.iter().map(|c| c.count).max().unwrap_or(1);
    let current = active().filter(|&i| i < count);
    let now = current.map(|i| EventDatum::category(i, bins[i].title.clone(), bins[i].total));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let text = current
        .map(|i| {
            let c = &bins[i];
            format!(
                "{}: {} {}, total {}",
                c.title,
                c.count,
                if c.count == 1 { "point" } else { "points" },
                fmt(c.total)
            )
        })
        .unwrap_or_default();
    // `y` is only the `data-vertical` flag (1.0 = below), since the style places the tooltip.
    let tip = current.map(|i| {
        let c = &bins[i];
        let (fx, fy) = (c.x / l.width, c.y / l.height);
        let tip = Tip {
            x: fx,
            y: if fy < 0.25 { 1.0 } else { 0.0 },
            title: c.title.clone(),
            rows: vec![TipRow::new(None, "Points", c.count.to_string()), TipRow::new(None, "Total", fmt(c.total))],
        };
        let swatch = format!("color-mix(in oklab, {} {:.0}%, var(--surface-sunken))", colour(0), 15.0 + c.t * 85.0);
        (tip, swatch, format!("left: {:.2}%; top: {:.2}%;", fx * 100.0, fy * 100.0))
    });
    let view_box = format!("0 0 {} {:.2}", l.width, l.height);

    rsx! {
        figure { "data-scope": "hexbin-map-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "{view_box}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through hexagons.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onclick: move |_| select_active(Some(datum), on_select),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            select_active(Some(datum), on_select);
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
                    for (i , c) in bins.iter().enumerate() {
                        path {
                            key: "{i}",
                            "data-part": "hex",
                            "data-active": (current == Some(i)).then_some(""),
                            d: "{c.path}",
                            style: "--t: {c.t:.3}; --i: {i.min(40)};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                        }
                    }
                }
                if let Some((tip, swatch, style)) = tip {
                    ChartTip { tip, swatch, style }
                }
            }
            div { "data-part": "scale", "aria-hidden": "true",
                span { "1" }
                span { "data-part": "ramp" }
                span { "{most}" }
                span { "data-part": "scale-label", "points per hexagon" }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{text}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Hexagon" }
                        th { scope: "col", "Points" }
                        th { scope: "col", "Total" }
                        th { scope: "col", "Includes" }
                    }
                }
                tbody {
                    for (i , c) in bins.iter().enumerate() {
                        tr {
                            th { scope: "row", "Hexagon {i + 1}" }
                            td { "{c.count}" }
                            td { {fmt(c.total)} }
                            td { "{c.names}" }
                        }
                    }
                }
            }
        }
    }
}
