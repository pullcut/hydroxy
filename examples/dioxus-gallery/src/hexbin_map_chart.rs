use dioxus::prelude::*;
use hydroxy_dioxus::hexbin_map_chart::{GeoPoint, HexbinMapChart, Projection, Region};

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

#[component]
pub fn HexbinMapChartDemo() -> Element {
    rsx! {
        HexbinMapChart {
            points: sites(),
            regions: uk(),
            label: "Public charging sites",
            radius: 14.0,
            projection: Projection::Mercator,
        }
    }
}
