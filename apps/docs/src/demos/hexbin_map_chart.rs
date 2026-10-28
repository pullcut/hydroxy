use leptos::prelude::*;
use hydroxy_leptos::hexbin_map_chart::*;

/// A rough outline of Great Britain and Northern Ireland as (longitude, latitude).
fn uk() -> Vec<Region> {
    let gb = [
        (-5.7, 50.0), (-3.0, 50.6), (1.4, 51.2), (1.7, 52.7), (0.3, 53.4), (-0.1, 54.5), (-1.6, 55.6), (-2.1, 57.7),
        (-3.0, 58.6), (-5.0, 58.6), (-6.2, 56.8), (-5.6, 55.3), (-4.9, 54.8), (-3.4, 54.9), (-3.0, 53.4), (-4.6, 53.3),
        (-4.2, 52.3), (-5.3, 51.7), (-3.3, 51.4),
    ];
    let ni = [(-8.2, 54.4), (-7.3, 55.3), (-6.0, 55.2), (-5.4, 54.4), (-6.3, 54.0), (-7.6, 54.1)];
    vec![Region::new("gb", "Great Britain", f64::NAN, gb), Region::new("ni", "Northern Ireland", f64::NAN, ni)]
}

/// Charging sites scattered around towns: more around bigger towns, each with a few chargers.
fn sites() -> Vec<GeoPoint> {
    let towns = [
        ("London", (-0.13, 51.51), 40), ("Birmingham", (-1.89, 52.49), 16), ("Manchester", (-2.24, 53.48), 18),
        ("Leeds", (-1.55, 53.80), 10), ("Glasgow", (-4.25, 55.86), 12), ("Edinburgh", (-3.19, 55.95), 9),
        ("Bristol", (-2.59, 51.45), 9), ("Cardiff", (-3.18, 51.48), 5), ("Newcastle", (-1.61, 54.97), 6),
        ("Norwich", (1.30, 52.63), 4), ("Aberdeen", (-2.09, 57.15), 3), ("Belfast", (-5.93, 54.60), 5),
    ];
    // A fixed pseudo-random spread, so the demo looks the same every time.
    let mut seed = 7_u32;
    let mut next = move || {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        f64::from(seed >> 8) / f64::from(1_u32 << 24) - 0.5
    };
    towns
        .iter()
        .flat_map(|&(town, (lon, lat), n)| {
            (0..n).map(|k| GeoPoint::new(format!("{town} site {}", k + 1), (lon + next() * 1.2, lat + next() * 0.7), (4.0 + next() * 6.0).round())).collect::<Vec<_>>()
        })
        .collect()
}

pub fn demo() -> AnyView {
    view! {
        <HexbinMapChart points=sites() regions=uk() label="Public charging sites" radius=14.0 projection=Projection::Mercator />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "hexbin-map-chart",
    name: "Hexbin map",
    group: "Maps",
    summary: "Points binned into hexagons over projected map shapes, each hexagon shaded by how many points fall inside. Reads density where individual dots would pile up.",
    demo,
    leptos: include_str!("hexbin_map_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/hexbin_map_chart.rs"),
    parts: &[
        &["HexbinMapChart", "root", "figure; `data-active` while a hexagon is picked out"],
        &["", "marks", "Focusable SVG; arrow keys step through hexagons from the top left"],
        &["", "land", "The base shapes, drawn plain"],
        &["", "hex", "One occupied hexagon, shaded by its point count through `--t`"],
        &["", "tooltip", "Points inside, their count and their summed value"],
        &["", "scale", "Colour ramp from one point to the fullest hexagon"],
        &["", "sr-table", "Every hexagon with its count, total and the names inside"],
    ],
    keys: &[
        &["Arrow keys", "Step through hexagons"],
        &["Home / End", "First or last hexagon"],
        &["Escape", "Clear the active hexagon"],
    ],
    props: &[
        &["points", "Vec<GeoPoint>", "Required. GeoPoint::new(name, (lon, lat), value); values sum into each hexagon's total"],
        &["regions", "Vec<Region>", "Base shapes drawn under the hexagons; their values are ignored"],
        &["label", "String", "Required. What the map shows; the accessible name"],
        &["radius", "f64", "Hexagon radius in viewBox units (640 wide). Default 16"],
        &["projection", "Projection", "Equirectangular (default), Mercator or Custom(fn)"],
        &["format", "Fn(f64) -> String", "Formats each hexagon's total"],
    ],
};
