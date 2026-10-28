use dioxus::prelude::*;
use oxide_dioxus::chart_frame::ChartFrame;
use oxide_dioxus::quiver_chart::{QuiverChart, VectorField};

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

#[component]
pub fn QuiverChartDemo() -> Element {
    let storm = VectorField::from_fn(VectorField::steps(-240.0, 240.0, 17), VectorField::steps(-160.0, 160.0, 13), cyclone);
    // 64 × 40 points: enough to draw on a canvas.
    let ocean = VectorField::from_fn(VectorField::steps(0.0, 2000.0, 64), VectorField::steps(0.0, 1000.0, 40), gyre);

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Wind around a low", description: "Arrows scale with wind speed; faint lines trace the flow.",
                QuiverChart {
                    field: storm,
                    label: "Surface wind around a low-pressure centre",
                    streamlines: 16,
                    format: move |v: f64| format!("{v:.0} km"),
                    magnitude_format: move |v: f64| format!("{v:.1} m/s"),
                }
            }
            ChartFrame { title: "Double gyre", description: "Equal-length arrows; colour shows current speed.",
                QuiverChart {
                    field: ocean,
                    label: "Ocean currents in a double gyre",
                    scaled: false,
                    height: 260,
                    format: move |v: f64| format!("{v:.0} km"),
                    magnitude_format: move |v: f64| format!("{v:.2} m/s"),
                }
            }
        }
    }
}
