use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::quiver_chart::*;

/// Surface wind in m/s around a low centred at the origin, positions in km: it circles
/// anticlockwise, fastest 80 km out, and spirals inwards.
fn cyclone(x: f64, y: f64) -> (f64, f64) {
    let r = x.hypot(y).max(1.0);
    let speed = 32.0 * (r / 80.0) * (1.0 - r / 80.0).exp();
    let (tx, ty) = (-y / r, x / r);
    let (ix, iy) = (-x / r, -y / r);
    (speed * (tx * 0.94 + ix * 0.34), speed * (ty * 0.94 + iy * 0.34))
}

/// A wind-driven double gyre in an ocean basin, currents in m/s over a 2000 × 1000 km box.
fn gyre(x: f64, y: f64) -> (f64, f64) {
    use std::f64::consts::PI;
    let (a, b) = (PI * x / 1000.0, PI * y / 1000.0);
    (-0.6 * a.sin() * b.cos(), 0.6 * a.cos() * b.sin())
}

pub fn demo() -> AnyView {
    let storm = VectorField::from_fn(VectorField::steps(-240.0, 240.0, 17), VectorField::steps(-160.0, 160.0, 13), cyclone);
    // 64 × 40 points: enough to draw on a canvas.
    let ocean = VectorField::from_fn(VectorField::steps(0.0, 2000.0, 64), VectorField::steps(0.0, 1000.0, 40), gyre);

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Wind around a low" description="Arrows scale with wind speed; faint lines trace the flow.">
                <QuiverChart
                    field=storm
                    label="Surface wind around a low-pressure centre"
                    streamlines=16
                    format=|v: f64| format!("{v:.0} km")
                    magnitude_format=|v: f64| format!("{v:.1} m/s")
                />
            </ChartFrame>
            <ChartFrame title="Double gyre" description="Equal-length arrows; colour shows current speed.">
                <QuiverChart
                    field=ocean
                    label="Ocean currents in a double gyre"
                    scaled=false
                    height=260
                    format=|v: f64| format!("{v:.0} km")
                    magnitude_format=|v: f64| format!("{v:.2} m/s")
                />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "quiver-chart",
    name: "Quiver chart",
    group: "Points",
    summary: "Show a vector field such as wind or current. An arrow at each grid point shows direction, with length and colour for strength, and streamlines can trace the flow. Large grids draw on a canvas.",
    demo,
    leptos: include_str!("quiver_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/quiver_chart.rs"),
    parts: &[
        &["QuiverChart", "root", "figure with a screen-reader table of position, components, magnitude and direction"],
        &["", "arrow", "One per grid point, growing from its base; data-still for zero vectors"],
        &["", "streamline", "Lines traced through the field (streamlines)"],
        &["", "canvas", "Grids of more than 2,000 points draw here instead of as arrows"],
        &["", "scale", "Magnitude colour ramp, after the figure"],
        &["", "tooltip", "Magnitude, direction and components at the nearest grid point"],
    ],
    keys: &[
        &["Arrow keys", "Move across the grid"],
        &["Home / End", "First or last point in the row"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["field", "VectorField", "Required. VectorField::from_fn(xs, ys, |x, y| (u, v)), or xs, ys, u, v directly"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["scaled", "bool", "Arrow length follows magnitude. Default true"],
        &["streamlines", "usize", "About how many streamlines to trace. Default 0"],
        &["height", "u16", "Plot height in px. Default 300"],
        &["format", "Fn(f64) -> String", "Formats x and y positions"],
        &["magnitude_format", "Fn(f64) -> String", "Formats magnitudes and components"],
    ],
};
