use dioxus::prelude::*;
use hydroxy_dioxus::cartogram_chart::{CartogramChart, CartogramKind, Projection, Region};
use hydroxy_dioxus::chart_frame::ChartFrame;

/// Rough outlines of the eleven western US states as (longitude, latitude), with their
/// 2023 populations in millions.
fn west() -> Vec<Region> {
    let state = |id: &str, name: &str, people: f64, ring: &[(f64, f64)]| Region::new(id, name, people, ring.to_vec());
    vec![
        state(
            "WA",
            "Washington",
            7.81,
            &[
                (-124.7, 48.4),
                (-123.0, 49.0),
                (-117.0, 49.0),
                (-117.0, 46.0),
                (-119.0, 45.9),
                (-121.2, 45.6),
                (-123.0, 46.2),
                (-124.0, 46.3),
            ],
        ),
        state(
            "OR",
            "Oregon",
            4.23,
            &[
                (-124.0, 46.3),
                (-123.0, 46.2),
                (-121.2, 45.6),
                (-119.0, 45.9),
                (-117.0, 46.0),
                (-116.9, 45.0),
                (-117.2, 44.3),
                (-117.0, 42.0),
                (-124.2, 42.0),
                (-124.5, 43.0),
            ],
        ),
        state(
            "CA",
            "California",
            38.97,
            &[
                (-124.2, 42.0),
                (-120.0, 42.0),
                (-120.0, 39.0),
                (-114.6, 35.0),
                (-114.7, 32.7),
                (-117.1, 32.5),
                (-118.5, 34.0),
                (-120.6, 34.6),
                (-122.5, 37.5),
                (-123.8, 39.8),
            ],
        ),
        state("NV", "Nevada", 3.19, &[(-120.0, 42.0), (-114.0, 42.0), (-114.0, 36.2), (-114.6, 35.0), (-120.0, 39.0)]),
        state(
            "ID",
            "Idaho",
            1.96,
            &[
                (-117.0, 49.0),
                (-116.0, 49.0),
                (-116.0, 47.9),
                (-114.3, 46.6),
                (-113.9, 45.7),
                (-111.0, 44.5),
                (-111.0, 42.0),
                (-117.0, 42.0),
                (-117.2, 44.3),
                (-116.9, 45.0),
                (-117.0, 46.0),
            ],
        ),
        state(
            "MT",
            "Montana",
            1.13,
            &[
                (-116.0, 49.0),
                (-104.0, 49.0),
                (-104.0, 45.0),
                (-111.0, 45.0),
                (-111.0, 44.5),
                (-113.9, 45.7),
                (-114.3, 46.6),
                (-116.0, 47.9),
            ],
        ),
        state("WY", "Wyoming", 0.58, &[(-111.0, 45.0), (-104.0, 45.0), (-104.0, 41.0), (-111.0, 41.0)]),
        state(
            "UT",
            "Utah",
            3.42,
            &[(-114.0, 42.0), (-111.0, 42.0), (-111.0, 41.0), (-109.0, 41.0), (-109.0, 37.0), (-114.0, 37.0)],
        ),
        state("CO", "Colorado", 5.88, &[(-109.0, 41.0), (-102.0, 41.0), (-102.0, 37.0), (-109.0, 37.0)]),
        state(
            "AZ",
            "Arizona",
            7.43,
            &[
                (-114.0, 37.0),
                (-109.0, 37.0),
                (-109.0, 31.3),
                (-111.0, 31.3),
                (-114.8, 32.5),
                (-114.6, 35.0),
                (-114.0, 36.2),
            ],
        ),
        state(
            "NM",
            "New Mexico",
            2.11,
            &[(-109.0, 37.0), (-103.0, 37.0), (-103.0, 32.0), (-106.6, 32.0), (-108.2, 31.8), (-109.0, 31.3)],
        ),
    ]
}

#[component]
pub fn CartogramChartDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: 24px;",
            ChartFrame { title: "Population of the western states", description: "Each circle's area is the state's 2023 population, placed as near its home as the neighbours allow.",
                CartogramChart {
                    regions: west(),
                    label: "Population of the western US states, 2023",
                    measure: "People (m)",
                    projection: Projection::Mercator,
                    format: |v: f64| format!("{v:.1}m"),
                }
            }
            ChartFrame { title: "Same data, non-contiguous", description: "Each state shrinks about its centre until its area matches its population; California, the densest, keeps its size.",
                CartogramChart {
                    regions: west(),
                    kind: CartogramKind::NonContiguous,
                    label: "Population of the western US states as a non-contiguous cartogram, 2023",
                    measure: "People (m)",
                    projection: Projection::Mercator,
                    format: |v: f64| format!("{v:.1}m"),
                }
            }
        }
    }
}
