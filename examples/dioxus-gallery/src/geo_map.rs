use dioxus::prelude::*;
use hydroxy_dioxus::geo_map::{GeoMap, GeoPoint, Projection, Region};

/// A tilted view: rotate the map 20° so the patchwork reads on a diagonal.
fn tilted((lon, lat): (f64, f64)) -> (f64, f64) {
    let (s, c) = 20f64.to_radians().sin_cos();
    (lon * c - lat * s, lon * s + lat * c)
}

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
    let offices = vec![GeoPoint::new("Head office", (14.0, 15.0), 240.0), GeoPoint::new("Harbour depot", (3.0, 4.0), 35.0).in_group(1)];
    // A polygon map: each sales territory keeps its own colour instead of a value shade.
    let territories = vec![
        cell(0, 0, "Southwick, South team", 12.0).with_colour(0),
        cell(0, 1, "Eastmere, South team", 9.0).with_colour(0),
        cell(0, 2, "Highfold, East team", 14.0).with_colour(1),
        cell(1, 0, "Westmark, West team", 11.0).with_colour(2),
        cell(1, 1, "Midvale, West team", 7.0).with_colour(2),
        cell(1, 2, "Northcote, East team", 10.0).with_colour(1),
    ];

    rsx! {
        div { style: "display: grid; gap: 32px;",
            GeoMap {
                regions,
                points: offices,
                label: "Customers per 1,000 people",
                measure: "Customers per 1,000",
                projection: Projection::Mercator,
            }
            GeoMap {
                regions: territories,
                label: "Sales territories",
                measure: "Open deals",
                projection: Projection::Custom(tilted),
            }
        }
    }
}
