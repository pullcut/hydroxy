use leptos::prelude::*;
use oxide_leptos::bubble_map_chart::*;

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

pub fn demo() -> AnyView {
    let city = |name: &str, at: (f64, f64), orders: f64| GeoPoint::new(name, at, orders);
    let points = vec![
        city("London", (-0.13, 51.51), 4820.0),
        city("Birmingham", (-1.89, 52.49), 1310.0),
        city("Manchester", (-2.24, 53.48), 1580.0),
        city("Leeds", (-1.55, 53.80), 920.0),
        city("Glasgow", (-4.25, 55.86), 1040.0),
        city("Edinburgh", (-3.19, 55.95), 860.0),
        city("Bristol", (-2.59, 51.45), 740.0),
        city("Cardiff", (-3.18, 51.48), 410.0),
        city("Belfast", (-5.93, 54.60), 380.0),
        city("Newcastle", (-1.61, 54.97), 520.0),
        // Pickup points: a second series in the second colour.
        city("Milton Keynes", (-0.76, 52.04), 640.0).in_group(1),
        city("Daventry", (-1.16, 52.26), 890.0).in_group(1),
        city("Warrington", (-2.59, 53.39), 470.0).in_group(1),
        city("Bellshill", (-4.02, 55.82), 300.0).in_group(1),
    ];
    let groups = vec!["Stores".to_string(), "Pickup points".to_string()];

    view! {
        <BubbleMapChart
            points=points
            regions=uk()
            groups=groups
            label="Weekly orders by location"
            measure="Orders"
            projection=Projection::Mercator
        />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bubble-map-chart",
    name: "Bubble map",
    group: "Maps",
    summary: "Proportional circles at longitude and latitude over base map shapes you supply. A circle's area is its value, and big circles draw first so small ones stay visible.",
    demo,
    leptos: include_str!("bubble_map_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bubble_map_chart.rs"),
    parts: &[
        &["BubbleMapChart", "root", "figure; `data-active` while a bubble is picked out"],
        &["", "marks", "Focusable SVG; arrow keys step through the points in the order given"],
        &["", "land", "The base shapes, drawn plain"],
        &["", "bubble", "Circle sized by value and coloured by group; `data-active` when picked out"],
        &["", "tooltip", "Name, value and series of the active bubble"],
        &["", "legend", "Series names, when there are two or more"],
        &["", "sr-table", "Every point with its coordinates and value"],
    ],
    keys: &[
        &["Arrow keys", "Step through the points"],
        &["Home / End", "First or last point"],
        &["Escape", "Clear the active point"],
    ],
    props: &[
        &["points", "Vec<GeoPoint>", "Required. GeoPoint::new(name, (lon, lat), value); .in_group(n) for the series colour"],
        &["regions", "Vec<Region>", "Base shapes drawn under the bubbles; their values are ignored"],
        &["label", "String", "Required. What the map shows; the accessible name"],
        &["measure", "String", "Required. Name of the value; heads the table column"],
        &["groups", "Vec<String>", "Series names by group index, for the legend and tooltip"],
        &["projection", "Projection", "Equirectangular (default), Mercator or Custom(fn)"],
        &["max_radius", "f64", "Radius of the largest bubble in viewBox units (640 wide). Default 28"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip and table"],
    ],
};
