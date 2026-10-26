//! `DonutChart`: parts of a whole as a ring with a legend. Geometry comes from
//! `oxide_charts::donut`. More than six parts fold into "Other".

use dioxus::prelude::*;
use oxide_charts::{donut, fold_other, format_value};

pub use oxide_charts::Datum;

/// Colour for slot `i`: the categorical order, with a folded "Other" in neutral grey.
fn slot_colour(i: usize, folded_other: bool, len: usize) -> String {
    if folded_other && i + 1 == len {
        "var(--chart-muted)".into()
    } else {
        format!("var(--chart-{})", i % 6 + 1)
    }
}

/// A ring chart with a total in the middle and a legend with values and shares.
#[component]
pub fn DonutChart(
    /// The parts, in display order.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Tickets by channel". Used as the accessible name.
    label: String,
    /// Ring diameter in px.
    #[props(default = 180)]
    size: u16,
    /// Ring width in px.
    #[props(default = 22)]
    thickness: u16,
    /// Small caption under the centre figure, such as "Tickets".
    center_label: Option<String>,
    /// Centre figure. Default: the formatted total.
    center_value: Option<String>,
    /// Formats values for the centre, legend and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let raw = data();
    let folded = raw.len() > 6;
    let parts = fold_other(&raw, 6);
    let n = parts.len();
    let values: Vec<f64> = parts.iter().map(|d| d.value).collect();
    let segs = donut::segments(&values, f64::from(size), f64::from(thickness), 2.0);
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let ring = donut::ring(f64::from(size), f64::from(thickness));
    let centre = center_value.unwrap_or_else(|| fmt(total));

    rsx! {
        figure { "data-scope": "donut-chart", "data-part": "root", "aria-label": "{label}", style: "--ox-segments: {n};",
            div { "data-part": "ring", style: "width: {size}px; height: {size}px;", "aria-hidden": "true",
                svg { view_box: "0 0 {size} {size}", width: "{size}", height: "{size}",
                    // Keyed by label: a changed share reshapes its own arc; new parts sweep in.
                    for (i, (d, s)) in parts.iter().zip(segs.iter()).enumerate() {
                        path {
                            key: "{d.label}",
                            "data-part": "segment",
                            "data-index": "{i}",
                            style: "--i: {i};",
                            d: "{s.path}",
                            stroke: slot_colour(i, folded, n),
                            stroke_width: "{ring.stroke_width}",
                            "pathLength": "1",
                        }
                    }
                }
                div { "data-part": "center",
                    span { "data-part": "center-value", "{centre}" }
                    if let Some(l) = center_label {
                        span { "data-part": "center-label", "{l}" }
                    }
                }
            }
            ul { "data-part": "legend",
                for (i, (d, s)) in parts.iter().zip(segs.iter()).enumerate() {
                    li { "data-part": "legend-item", "data-index": "{i}",
                        span { "data-part": "swatch", style: "background: {slot_colour(i, folded, n)};" }
                        span { "data-part": "legend-label", "{d.label}" }
                        span { "data-part": "legend-value", {fmt(d.value)} }
                        span { "data-part": "legend-share", {format!("{:.0}%", s.fraction * 100.0)} }
                    }
                }
            }
        }
    }
}
