//! `GaugeChart` and `RadialBarChart`. Pie charts are covered by `DonutChart`.
//! Geometry comes from `oxide_charts::radial`.

use dioxus::prelude::*;
use oxide_charts::{format_value, radial};

pub use oxide_charts::Datum;
pub use oxide_charts::radial::Threshold;

fn share(value: f64, top: f64) -> f64 {
    if top > 0.0 { (value / top * 100.0).clamp(0.0, 100.0) } else { 0.0 }
}

/// One value on a three-quarter arc. Thresholds colour the arc by the band the value
/// reaches and are marked on the track.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn GaugeChart(
    /// The current value.
    value: f64,
    /// Lowest value on the scale.
    #[props(default = 0.0)]
    min: f64,
    /// Highest value on the scale.
    #[props(default = 100.0)]
    max: f64,
    /// What is measured, such as "Support SLA met". Used as the accessible name.
    label: String,
    /// Colour bands: `Threshold::new(from, colour)`. Default: everything in `--chart-1`.
    #[props(default)]
    thresholds: Vec<Threshold>,
    /// Text under the centre figure, such as "this week".
    caption: Option<String>,
    /// Diameter in px.
    #[props(default = 200)]
    size: u16,
    /// Arc width in px.
    #[props(default = 16)]
    thickness: u16,
    /// Formats the value and the scale ends. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (box_px, arc_px) = (f64::from(size), f64::from(thickness));
    let centre = box_px / 2.0;
    let g = radial::gauge(value, min, max, arc_px, box_px, &thresholds);
    // Scale ends sit just below the arc's two ends, 45 degrees either side of straight down.
    let end_dx = g.radius * std::f64::consts::FRAC_1_SQRT_2;
    let end_y = centre + end_dx + arc_px / 2.0 + 4.0;
    let colour = g.colour.clone().unwrap_or_else(|| "var(--chart-1)".into());
    let hidden = if g.fraction <= 0.0 { "opacity: 0;" } else { "" };

    rsx! {
        div {
            "data-scope": "gauge-chart",
            "data-part": "root",
            role: "meter",
            "aria-label": "{label}",
            "aria-valuemin": "{min}",
            "aria-valuemax": "{max}",
            "aria-valuenow": "{value}",
            "aria-valuetext": fmt(value),
            style: "width: {size}px; height: {end_y + 14.0}px;",
            svg { width: "{size}", height: "{size}", view_box: "0 0 {size} {size}", "aria-hidden": "true",
                path { "data-part": "track", d: "{g.track}", style: "stroke-width: {thickness}px;" }
                path {
                    "data-part": "value-arc",
                    d: "{g.track}",
                    "pathLength": "1",
                    style: "stroke-width: {thickness}px; stroke: {colour}; stroke-dasharray: {g.fraction:.4} 1; {hidden}",
                }
                for (i , (x1 , y1 , x2 , y2)) in g.marks.iter().enumerate() {
                    line {
                        key: "{i}",
                        "data-part": "mark",
                        x1: "{x1}",
                        y1: "{y1}",
                        x2: "{x2}",
                        y2: "{y2}",
                    }
                }
            }
            div { "data-part": "center", style: "top: {centre}px;",
                span { "data-part": "center-value", {fmt(value)} }
                if let Some(t) = caption {
                    span { "data-part": "center-label", "{t}" }
                }
            }
            span { "data-part": "end", "aria-hidden": "true", style: "left: {centre - end_dx}px; top: {end_y}px;", {fmt(min)} }
            span { "data-part": "end", "aria-hidden": "true", style: "left: {centre + end_dx}px; top: {end_y}px;", {fmt(max)} }
        }
    }
}

/// Concentric progress rings, the first outermost, each filled against `max`.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RadialBarChart(
    /// The rings, in display order and categorical colour order. Up to six read well.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Goal progress". Used as the accessible name.
    label: String,
    /// What a full ring means. Default: the largest value.
    max: Option<f64>,
    /// Diameter in px.
    #[props(default = 200)]
    size: u16,
    /// Ring width in px.
    #[props(default = 14)]
    thickness: u16,
    /// Formats values for the legend and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = data();
    let top = max.unwrap_or_else(|| items.iter().map(|d| d.value).filter(|v| v.is_finite()).fold(0.0, f64::max));
    let values: Vec<f64> = items.iter().map(|d| d.value).collect();
    let rings = radial::rings(&values, top, f64::from(size), f64::from(thickness), 6.0);

    let fill_styles: Vec<String> = rings
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let hidden = if r.fraction <= 0.0 { " opacity: 0;" } else { "" };
            format!("stroke-width: {thickness}px; stroke: var(--chart-{}); stroke-dasharray: {:.4} 1;{hidden}", i % 6 + 1, r.fraction)
        })
        .collect();

    rsx! {
        figure { "data-scope": "radial-bar-chart", "data-part": "root",
            svg {
                "data-part": "rings",
                width: "{size}",
                height: "{size}",
                view_box: "0 0 {size} {size}",
                "aria-hidden": "true",
                for (i , r) in rings.iter().enumerate() {
                    g { key: "{i}", "data-part": "ring", "data-index": "{i}", style: "--i: {i};",
                        path { "data-part": "track", d: "{r.path}", style: "stroke-width: {thickness}px;" }
                        path {
                            "data-part": "fill",
                            d: "{r.path}",
                            "pathLength": "1",
                            style: "{fill_styles[i]}",
                        }
                    }
                }
            }
            ul { "data-part": "legend", "aria-hidden": "true",
                for (i , d) in items.iter().enumerate() {
                    li { key: "{d.label}", "data-part": "legend-item", "data-index": "{i}",
                        span { "data-part": "swatch", style: "background: var(--chart-{i % 6 + 1});" }
                        span { "{d.label}" }
                        span { "data-part": "legend-value", {fmt(d.value)} }
                        span { "data-part": "legend-share",
                            "{share(d.value, top):.0}%"
                        }
                    }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                tbody {
                    for d in items.iter() {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                            td { "of {fmt(top)}" }
                        }
                    }
                }
            }
        }
    }
}
