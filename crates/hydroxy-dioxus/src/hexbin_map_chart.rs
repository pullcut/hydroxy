//! `HexbinMapChart`: points binned into hexagons over projected map shapes, each
//! hexagon shaded by how many points it holds. Geometry comes from `hydroxy_charts::geo`
//! and `hydroxy_charts::hexbin`; no map data is bundled.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, geo, hexbin, nav};

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Hexagons over a map, darker where more points fall. Hover, touch or focus the map and
/// use the arrow keys to step through the hexagons from the top left, Home and End to
/// jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
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
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let pts = points();
    let l = geo::layout(&regions(), &pts, projection);
    let bins = hexbin::cells(&pts, &l.points, radius);
    let count = bins.len();
    let most = bins.iter().map(|c| c.count).max().unwrap_or(1);
    let current = active().filter(|&i| i < count);
    let text = current
        .map(|i| {
            let c = &bins[i];
            format!("{}: {} {}, total {}", c.title, c.count, if c.count == 1 { "point" } else { "points" }, fmt(c.total))
        })
        .unwrap_or_default();
    let tip = current.map(|i| (&bins[i], bins[i].x / l.width * 100.0, bins[i].y / l.height * 100.0));
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
                    onkeydown: move |e| {
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
                if let Some((c , left , top)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if left > 50.0 { "left" } else { "right" },
                        "data-vertical": if top < 25.0 { "below" } else { "above" },
                        style: "left: {left:.2}%; top: {top:.2}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: color-mix(in oklab, var(--chart-1) {15.0 + c.t * 85.0:.0}%, var(--surface-sunken));" }
                            "{c.title}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Points" }
                                span { "data-part": "tooltip-value", "{c.count}" }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Total" }
                                span { "data-part": "tooltip-value", {fmt(c.total)} }
                            }
                        }
                    }
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
