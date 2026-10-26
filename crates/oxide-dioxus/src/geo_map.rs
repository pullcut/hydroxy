//! `GeoMap`: a choropleth over caller-supplied polygons. Geometry comes from
//! `oxide_charts::geo`; no map data is bundled. Regions take hover and focus.

use dioxus::prelude::*;
use oxide_charts::{format_value, geo};

pub use oxide_charts::geo::{Projection, Region};

/// Regions shaded on a sequential scale by value. Hover or focus a region to read it.
#[component]
pub fn GeoMap(
    /// The regions, as rings of `(longitude, latitude)` degrees.
    regions: ReadSignal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Name of the measure, such as "Customers per 1k people". Heads the table column.
    measure: String,
    /// Equirectangular (default) or Mercator.
    #[props(default)]
    projection: Projection,
    /// Formats values on the legend, readout and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let items = regions();
    let l = geo::layout(&items, projection);
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let text = move |r: &Region| format!("{}: {}", r.name, if r.value.is_finite() { fmt(r.value) } else { "no data".to_string() });
    let readout = active().and_then(|i| items.get(i)).map_or_else(|| "Hover or focus a region".to_string(), text);

    rsx! {
        figure { "data-scope": "geo-map", "data-part": "root",
            svg { "data-part": "marks", view_box: "{view_box}", role: "group", "aria-label": "{label}",
                for (i , (r , s)) in items.iter().zip(l.shapes.iter()).enumerate() {
                    path {
                        key: "{r.id}",
                        "data-part": "region",
                        "data-empty": s.t.is_none().then_some(""),
                        "data-active": (active() == Some(i)).then_some(""),
                        d: "{s.path}",
                        style: s.t.map(|t| format!("--t: {t:.3};")),
                        tabindex: "0",
                        role: "img",
                        "aria-label": text(r),
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerleave: move |_| active.set(None),
                        onfocus: move |_| active.set(Some(i)),
                        onblur: move |_| active.set(None),
                    }
                }
            }
            div { "data-part": "legend", "aria-hidden": "true",
                output { "data-part": "readout", "{readout}" }
                div { "data-part": "scale",
                    span { {fmt(l.range.0)} }
                    span { "data-part": "ramp" }
                    span { {fmt(l.range.1)} }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Region" }
                        th { scope: "col", "{measure}" }
                    }
                }
                tbody {
                    for r in items.iter() {
                        tr {
                            th { scope: "row", "{r.name}" }
                            td { if r.value.is_finite() { {fmt(r.value)} } else { "–" } }
                        }
                    }
                }
            }
        }
    }
}
