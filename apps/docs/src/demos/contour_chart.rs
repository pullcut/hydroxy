use leptos::prelude::*;
use oxide_leptos::contour_chart::*;

/// Two overlapping hills on a 36 × 20 grid.
fn terrain() -> Vec<f64> {
    let hill = |x: f64, y: f64, cx: f64, cy: f64, h: f64, r: f64| h * (-((x - cx).powi(2) + (y - cy).powi(2)) / (2.0 * r * r)).exp();
    (0..20)
        .flat_map(|r| (0..36).map(move |c| (f64::from(c), f64::from(r))))
        .map(|(x, y)| hill(x, y, 11.0, 7.0, 80.0, 5.0) + hill(x, y, 25.0, 13.0, 60.0, 6.0))
        .collect()
}

pub fn demo() -> AnyView {
    view! {
        <div style="display: grid; gap: var(--space-10);">
            // Contour lines at nice levels, over a filled heat field.
            <ContourChart values=terrain() columns=36 label="Elevation in metres" filled=true format=|v: f64| format!("{v:.0} m") />
            // Lines alone.
            <ContourChart values=terrain() columns=36 label="Elevation contours" levels=8 />
            // A heat field alone.
            <HeatField values=terrain() columns=36 label="Elevation heat field" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "contour-chart",
    name: "Contour chart",
    group: "Charts",
    summary: "Show a measure that varies across a surface, as marching-squares contour lines, a filled heat field, or both. HeatField is the filled form on its own.",
    demo,
    leptos: include_str!("contour_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/contour_chart.rs"),
    parts: &[
        &["ContourChart", "root", "figure with a screen-reader table of the grid"],
        &["", "cell", "One square per value, shaded on a sequential scale (filled only)"],
        &["", "contour", "A line at one level; darker for higher levels"],
        &["", "scale", "Colour ramp with the smallest and largest value"],
    ],
    keys: &[],
    props: &[
        &["values", "Vec<f64>", "Required. Row-major grid; NaN leaves a gap"],
        &["columns", "usize", "Required. Values per row"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["levels", "usize", "About how many contour levels. Default 6"],
        &["filled", "bool", "Shade each cell under the lines (ContourChart only)"],
        &["format", "Fn(f64) -> String", "Formats the scale, tooltips and table"],
    ],
};
