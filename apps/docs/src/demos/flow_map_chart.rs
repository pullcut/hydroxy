use leptos::prelude::*;
use oxide_leptos::chart_frame::ChartFrame;
use oxide_leptos::flow_map_chart::*;

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

/// Airports, then one-way passengers a year on the busiest domestic routes, in thousands.
fn routes() -> (Vec<GeoPoint>, Vec<Flow>) {
    let airport = |name: &str, at: (f64, f64)| GeoPoint::new(name, at, 0.0);
    let places = vec![
        airport("London", (-0.45, 51.47)),
        airport("Edinburgh", (-3.37, 55.95)),
        airport("Glasgow", (-4.43, 55.87)),
        airport("Belfast", (-5.87, 54.62)),
        airport("Aberdeen", (-2.20, 57.20)),
        airport("Inverness", (-4.05, 57.54)),
        airport("Newcastle", (-1.69, 55.04)),
        airport("Manchester", (-2.27, 53.35)),
        airport("Bristol", (-2.72, 51.38)),
        airport("Southampton", (-1.36, 50.95)),
    ];
    let flows = vec![
        Flow::new(0, 3, 810.0),
        Flow::new(3, 0, 790.0),
        Flow::new(0, 1, 720.0),
        Flow::new(1, 0, 700.0),
        Flow::new(0, 2, 520.0),
        Flow::new(2, 0, 505.0),
        Flow::new(0, 4, 260.0),
        Flow::new(0, 5, 180.0),
        Flow::new(0, 6, 140.0),
        Flow::new(8, 1, 210.0),
        Flow::new(8, 3, 190.0),
        Flow::new(7, 4, 90.0),
        Flow::new(9, 2, 75.0),
    ];
    (places, flows)
}

pub fn demo() -> AnyView {
    let (places, flows) = routes();
    view! {
        <ChartFrame title="Domestic air passengers" description="One-way passengers a year on the busiest UK domestic routes. Arrows run from origin to destination.">
            <FlowMapChart
                places
                flows
                regions=uk()
                label="Domestic air passengers by route"
                measure="Passengers"
                projection=Projection::Mercator
                format=|v: f64| format!("{v:.0}k")
            />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "flow-map-chart",
    name: "Flow map",
    group: "Maps",
    summary: "Curved arrows between places over base map shapes you supply, as wide as their volume. Each arrow draws from origin to destination, and a flow and its return bow apart so both stay readable.",
    demo,
    leptos: include_str!("flow_map_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/flow_map_chart.rs"),
    parts: &[
        &["FlowMapChart", "root", "figure; `data-active` while a flow is picked out"],
        &["", "marks", "Focusable SVG; arrow keys step through the flows in the order given"],
        &["", "land", "The base shapes, drawn plain"],
        &["", "flow", "One flow's group; `data-active` when picked out, the rest fade back"],
        &["", "line", "The curve, width by volume; draws from origin to destination"],
        &["", "head", "Arrowhead at the destination"],
        &["", "hit", "A wider invisible stroke that takes the pointer"],
        &["", "place", "A place's dot and name; `data-end` is origin or destination for the active flow"],
        &["", "tooltip", "Route and volume of the active flow"],
        &["", "sr-table", "Every flow with its origin, destination and volume"],
    ],
    keys: &[
        &["Arrow keys", "Step through the flows"],
        &["Home / End", "First or last flow"],
        &["Escape", "Clear the active flow"],
    ],
    props: &[
        &["places", "Vec<GeoPoint>", "Required. GeoPoint::new(name, (lon, lat), _); only name and position are used"],
        &["flows", "Vec<Flow>", "Required. Flow::new(from, to, volume) with indices into places"],
        &["regions", "Vec<Region>", "Base shapes drawn under the flows; their values are ignored"],
        &["label", "String", "Required. What the map shows; the accessible name"],
        &["measure", "String", "Required. Name of the volume; heads the table column"],
        &["projection", "Projection", "Equirectangular (default), Mercator or Custom(fn)"],
        &["max_width", "f64", "Width of the biggest flow in viewBox units (640 wide). Default 12"],
        &["format", "Fn(f64) -> String", "Formats volumes in the tooltip and table"],
    ],
};
