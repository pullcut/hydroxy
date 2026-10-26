use leptos::prelude::*;
use oxide_leptos::geo_map::*;

pub fn demo() -> AnyView {
    // A made-up country: a 3 × 2 patchwork of regions that share their corner points.
    // Real maps pass their own rings of (longitude, latitude).
    let p = [
        [(0.0, 0.0), (11.0, -2.0), (19.0, 1.0), (30.0, 0.0)],
        [(-2.0, 10.0), (9.0, 11.0), (21.0, 9.0), (32.0, 11.0)],
        [(1.0, 20.0), (10.0, 22.0), (20.0, 19.0), (29.0, 21.0)],
    ];
    let cell = |row: usize, col: usize, name: &str, value: f64| {
        Region::new(name.to_lowercase(), name, value, [p[row][col], p[row][col + 1], p[row + 1][col + 1], p[row + 1][col]])
    };
    let regions = vec![
        cell(0, 0, "Southwick", 18.0),
        cell(0, 1, "Eastmere", 42.0),
        cell(0, 2, "Highfold", 27.0),
        cell(1, 0, "Westmark", 64.0),
        cell(1, 1, "Midvale", 91.0),
        // A region with no data stays unshaded rather than reading as zero.
        cell(1, 2, "Northcote", f64::NAN),
    ];

    view! {
        <GeoMap regions=regions label="Customers per 1,000 people" measure="Customers per 1,000" projection=Projection::Mercator />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "geo-map",
    name: "Geo map",
    group: "Charts",
    summary: "A choropleth over polygons you supply as longitude and latitude rings. No map data is bundled; bring your own shapes. Regions shade on a sequential scale and take hover or focus.",
    demo,
    leptos: include_str!("geo_map.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/geo_map.rs"),
    parts: &[
        &["GeoMap", "root", "figure with a screen-reader table of every region"],
        &["", "region", "Focusable path shaded by value; unshaded when there is no value"],
        &["", "readout", "Name and value of the hovered or focused region"],
        &["", "scale", "Colour ramp with the smallest and largest value"],
    ],
    keys: &[&["Tab", "Move between regions and read each one"]],
    props: &[
        &["regions", "Vec<Region>", "Required. Region::new(id, name, value, ring); more rings via .rings, even-odd filled"],
        &["label", "String", "Required. What the map shows; the accessible name"],
        &["measure", "String", "Required. Name of the value; heads the table column"],
        &["projection", "Projection", "Equirectangular (default) or Mercator"],
        &["format", "Fn(f64) -> String", "Formats the scale, readout and table"],
    ],
};
