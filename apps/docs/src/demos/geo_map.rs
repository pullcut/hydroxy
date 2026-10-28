use hydroxy_leptos::geo_map::*;
use leptos::prelude::*;

/// A tilted view: rotate the map 20° so the patchwork reads on a diagonal.
fn tilted((lon, lat): (f64, f64)) -> (f64, f64) {
    let (s, c) = 20f64.to_radians().sin_cos();
    (lon * c - lat * s, lon * s + lat * c)
}

pub fn demo() -> AnyView {
    // A made-up country: a 3 × 2 patchwork of regions that share their corner points.
    // Real maps pass their own rings of (longitude, latitude).
    let p = [
        [(0.0, 0.0), (11.0, -2.0), (19.0, 1.0), (30.0, 0.0)],
        [(-2.0, 10.0), (9.0, 11.0), (21.0, 9.0), (32.0, 11.0)],
        [(1.0, 20.0), (10.0, 22.0), (20.0, 19.0), (29.0, 21.0)],
    ];
    let cell = |row: usize, col: usize, name: &str, value: f64| {
        Region::new(
            name.to_lowercase(),
            name,
            value,
            [p[row][col], p[row][col + 1], p[row + 1][col + 1], p[row + 1][col]],
        )
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
    let offices = vec![
        GeoPoint::new("Head office", (14.0, 15.0), 240.0),
        GeoPoint::new("Harbour depot", (3.0, 4.0), 35.0).in_group(1),
    ];
    // A polygon map: each sales territory keeps its own colour instead of a value shade.
    let territories = vec![
        cell(0, 0, "Southwick, South team", 12.0).with_colour(0),
        cell(0, 1, "Eastmere, South team", 9.0).with_colour(0),
        cell(0, 2, "Highfold, East team", 14.0).with_colour(1),
        cell(1, 0, "Westmark, West team", 11.0).with_colour(2),
        cell(1, 1, "Midvale, West team", 7.0).with_colour(2),
        cell(1, 2, "Northcote, East team", 10.0).with_colour(1),
    ];

    view! {
        <div style="display: grid; gap: 32px;">
            <GeoMap
                regions=regions
                points=offices
                label="Customers per 1,000 people"
                measure="Customers per 1,000"
                projection=Projection::Mercator
            />
            <GeoMap regions=territories label="Sales territories" measure="Open deals" projection=Projection::Custom(tilted) />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "geo-map",
    name: "Geo map",
    group: "Maps",
    summary: "A choropleth or polygon map over polygons you supply as longitude and latitude rings, with optional points on top. No map data is bundled; bring your own shapes. Use a built-in projection or pass your own function.",
    demo,
    leptos: include_str!("geo_map.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/geo_map.rs"),
    parts: &[
        &["GeoMap", "root", "figure; `data-active` while a region or point is picked out"],
        &["", "marks", "Focusable SVG; arrow keys step through regions, then points"],
        &["", "region", "Path shaded by value, or filled with its own colour; `data-empty` when there is no value"],
        &["", "point", "Circle for a point, coloured by its group"],
        &["", "tooltip", "Name and value of the active region or point"],
        &["", "readout", "The same, as text under the map"],
        &["", "scale", "Colour ramp with the smallest and largest value; hidden on a polygon map"],
        &["", "sr-table", "Every region and point with its value, for screen readers and CSV export"],
    ],
    keys: &[
        &["Arrow keys", "Step through regions, then points"],
        &["Home / End", "First or last mark"],
        &["Escape", "Clear the active mark"],
    ],
    props: &[
        &[
            "regions",
            "Vec<Region>",
            "Required. Region::new(id, name, value, ring); .with_colour(slot) for a polygon map; more rings via .rings, even-odd filled",
        ],
        &[
            "points",
            "Vec<GeoPoint>",
            "GeoPoint::new(name, (lon, lat), value), drawn on top; .in_group(n) sets the colour",
        ],
        &["label", "String", "Required. What the map shows; the accessible name"],
        &["measure", "String", "Required. Name of the value; heads the table column"],
        &[
            "projection",
            "Projection",
            "Equirectangular (default), Mercator or Custom(fn((lon, lat)) -> (x, y)), y pointing north",
        ],
        &["format", "Fn(f64) -> String", "Formats the scale, tooltip, readout and table"],
    ],
};
