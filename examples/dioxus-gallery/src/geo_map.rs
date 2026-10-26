use dioxus::prelude::*;
use oxide_dioxus::geo_map::{GeoMap, Projection, Region};

#[component]
pub fn GeoMapDemo() -> Element {
    // A made-up country: a 3 x 2 patchwork of regions that share their corner points.
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
        cell(1, 2, "Northcote", f64::NAN),
    ];

    rsx! {
        GeoMap {
            regions,
            label: "Customers per 1,000 people",
            measure: "Customers per 1,000",
            projection: Projection::Mercator,
        }
    }
}
