//! `GaugeChart` and `RadialBarChart`. Pie charts are covered by `DonutChart`.
//! Geometry comes from `oxide_charts::radial`.

use dioxus::prelude::*;
use oxide_charts::{format_value, nav, radial};

pub use oxide_charts::Datum;
pub use oxide_charts::radial::Threshold;

fn share(value: f64, top: f64) -> f64 {
    if top > 0.0 { (value / top * 100.0).clamp(0.0, 100.0) } else { 0.0 }
}

/// One value on a three-quarter arc. Thresholds colour the arc by the band the value
/// reaches and are marked on the track. A changed value glides along the arc. Hover,
/// touch or focus the gauge to read the value and its band; Escape clears.
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
    /// Formats the value, the scale ends, tooltip and table. Default: whole numbers, else one decimal place.
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
    let (lo, hi) = radial::band(value, min, max, &thresholds);
    let band = format!("{}–{}", fmt(lo), fmt(hi));
    let band_name = if thresholds.is_empty() { "Scale" } else { "Band" };
    let mut active = use_signal(|| false);
    let (tx, ty) = radial::anchor(box_px, g.radius, radial::gauge_angle(g.fraction));
    let announcement = if active() { format!("{label}: {}, {band}", fmt(value)) } else { String::new() };

    rsx! {
        div {
            "data-scope": "gauge-chart",
            "data-part": "root",
            role: "meter",
            tabindex: "0",
            "aria-label": "{label}",
            "aria-valuemin": "{min}",
            "aria-valuemax": "{max}",
            "aria-valuenow": "{value}",
            "aria-valuetext": fmt(value),
            "data-active": active().then_some(""),
            style: "width: {size}px; height: {end_y + 14.0}px;",
            onpointerenter: move |_| active.set(true),
            onpointerdown: move |_| active.set(true),
            onpointerleave: move |_| active.set(false),
            onfocus: move |_| active.set(true),
            onblur: move |_| active.set(false),
            onkeydown: move |e| {
                if e.key() == Key::Escape && active() {
                    e.prevent_default();
                    active.set(false);
                }
            },
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
            div { "data-part": "center", style: "top: {centre}px;", "aria-hidden": "true",
                span { "data-part": "center-value", {fmt(value)} }
                if let Some(t) = caption {
                    span { "data-part": "center-label", "{t}" }
                }
            }
            span { "data-part": "end", "aria-hidden": "true", style: "left: {centre - end_dx}px; top: {end_y}px;", {fmt(min)} }
            span { "data-part": "end", "aria-hidden": "true", style: "left: {centre + end_dx}px; top: {end_y}px;", {fmt(max)} }
            if active() {
                div {
                    "data-part": "tooltip",
                    "data-anchored": "",
                    "data-side": if tx > 0.5 { "right" } else { "left" },
                    style: "left: {tx * box_px:.1}px; top: {ty * box_px:.1}px;",
                    "aria-hidden": "true",
                    div { "data-part": "tooltip-label",
                        span { "data-part": "swatch", style: "background: {colour};" }
                        "{label}"
                    }
                    ul {
                        li { "data-part": "tooltip-row",
                            span { "data-part": "tooltip-name", "Value" }
                            span { "data-part": "tooltip-value", {fmt(value)} }
                        }
                        li { "data-part": "tooltip-row",
                            span { "data-part": "tooltip-name", "{band_name}" }
                            span { "data-part": "tooltip-value", "{band}" }
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Measure" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "{band_name}" }
                    }
                }
                tbody {
                    tr {
                        th { scope: "row", "{label}" }
                        td { {fmt(value)} }
                        td { "{band}" }
                    }
                }
            }
        }
    }
}

/// Concentric progress rings, the first outermost, each filled against `max`. Changed
/// values glide. Hover, touch or focus a ring or legend row to read it; focus the rings
/// and use the arrow keys to step through them, Home and End to jump, Escape to clear.
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
    /// Formats values for the legend, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = data();
    let top = max.unwrap_or_else(|| items.iter().map(|d| d.value).filter(|v| v.is_finite()).fold(0.0, f64::max));
    let values: Vec<f64> = items.iter().map(|d| d.value).collect();
    let rings = radial::rings(&values, top, f64::from(size), f64::from(thickness), 6.0);
    let n = rings.len();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let announcement = current.map(|i| {
        let d = &items[i];
        format!("{}: {} of {}, {:.0}%", d.label, fmt(d.value), fmt(top), share(d.value, top))
    });
    let tip = current.map(|i| (i, radial::anchor(f64::from(size), rings[i].radius, rings[i].fraction * 360.0)));

    let fill_styles: Vec<String> = rings
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let hidden = if r.fraction <= 0.0 { " opacity: 0;" } else { "" };
            format!("stroke-width: {thickness}px; stroke: var(--chart-{}); stroke-dasharray: {:.4} 1;{hidden}", i % 6 + 1, r.fraction)
        })
        .collect();

    rsx! {
        figure { "data-scope": "radial-bar-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div {
                "data-part": "plot",
                style: "width: {size}px; height: {size}px;",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through rings.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg {
                    "data-part": "rings",
                    width: "{size}",
                    height: "{size}",
                    view_box: "0 0 {size} {size}",
                    "aria-hidden": "true",
                    for (i , r) in rings.iter().enumerate() {
                        g {
                            key: "{i}",
                            "data-part": "ring",
                            "data-index": "{i}",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--i: {i};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
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
                if let Some((i, (x, y))) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if x > 0.5 { "right" } else { "left" },
                        style: "left: {x * 100.0:.3}%; top: {y * 100.0:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: var(--chart-{i % 6 + 1});" }
                            "{items[i].label}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {fmt(items[i].value)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Of {fmt(top)}" }
                                span { "data-part": "tooltip-value", "{share(items[i].value, top):.0}%" }
                            }
                        }
                    }
                }
            }
            ul { "data-part": "legend", "aria-hidden": "true",
                for (i , d) in items.iter().enumerate() {
                    li {
                        key: "{d.label}",
                        "data-part": "legend-item",
                        "data-index": "{i}",
                        "data-active": (current == Some(i)).then_some(""),
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerleave: move |_| active.set(None),
                        span { "data-part": "swatch", style: "background: var(--chart-{i % 6 + 1});" }
                        span { "{d.label}" }
                        span { "data-part": "legend-value", {fmt(d.value)} }
                        span { "data-part": "legend-share",
                            "{share(d.value, top):.0}%"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Name" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Max" }
                        th { scope: "col", "Share" }
                    }
                }
                tbody {
                    for d in items.iter() {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                            td { {fmt(top)} }
                            td { "{share(d.value, top):.0}%" }
                        }
                    }
                }
            }
        }
    }
}
