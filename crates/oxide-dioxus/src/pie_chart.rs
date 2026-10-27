//! `PieChart`: parts of a whole as a full pie. Slices come from `oxide_charts::pie`,
//! which reuses the donut arcs with a ring as thick as the radius. More than six
//! parts fold into "Other".

use dioxus::prelude::*;
use oxide_charts::{fold_other, format_value, funnel, nav, pie};

pub use oxide_charts::Datum;
pub use oxide_charts::pie::Labels as PieLabels;

fn slot_colour(i: usize, folded_other: bool, len: usize) -> String {
    if folded_other && i + 1 == len { "var(--chart-muted)".into() } else { format!("var(--chart-{})", i % 6 + 1) }
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

fn share(f: f64) -> String {
    format!("{:.0}%", f * 100.0)
}

/// A full pie. Hover, touch or focus a slice to pull it out and read it; focus the pie
/// and use the arrow keys to step through slices, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn PieChart(
    /// The parts, in display order.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Revenue by region". Used as the accessible name.
    label: String,
    /// Pie diameter in px.
    #[props(default = 200)]
    size: u16,
    /// Where slice labels go. Default inside, as shares.
    #[props(default)]
    labels: PieLabels,
    /// Formats values for the tooltip, legend and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let raw = data();
    let folded = raw.len() > 6;
    let parts = fold_other(&raw, 6);
    let n = parts.len();
    let values: Vec<f64> = parts.iter().map(|d| d.value).collect();
    let layout = pie::layout(&values, f64::from(size), 1.5, labels);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let colour = |i: usize| slot_colour(i, folded, n);
    let announcement = current.map(|i| format!("{}: {}, {}", parts[i].label, fmt(parts[i].value), share(layout.slices[i].fraction))).unwrap_or_default();
    let (w, h, pad) = (layout.width, layout.height, layout.pad);

    rsx! {
        figure { "data-scope": "pie-chart", "data-part": "root", style: "--ox-segments: {n};",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through slices.",
                style: "width: {w}px; aspect-ratio: {w} / {h};",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    let key = e.key().to_string();
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 {w} {h}", "aria-hidden": "true",
                    g { transform: "translate({pad.0} {pad.1})",
                        // Keyed by label: a changed share reshapes its own arc; new parts sweep in.
                        for (i , (d , s)) in parts.iter().zip(layout.slices.iter()).enumerate() {
                            path {
                                key: "{d.label}",
                                "data-part": "slice",
                                "data-index": "{i}",
                                "data-active": (current == Some(i)).then_some(""),
                                style: "--i: {i}; --dx: {s.out.0 * 8.0:.2}px; --dy: {s.out.1 * 8.0:.2}px;",
                                d: "{s.path}",
                                stroke: "{colour(i)}",
                                stroke_width: "{layout.stroke_width}",
                                "pathLength": "1",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                            }
                        }
                    }
                    for (i , (d , s)) in parts.iter().zip(layout.slices.iter()).enumerate().filter(|(_, (_, s))| s.fraction > 0.0) {
                        if let Some(l) = s.leader.as_ref() {
                            g { "data-part": "leader", "data-index": "{i}", style: "--i: {i};",
                                path { d: "{l.path}" }
                                text {
                                    x: "{l.x}",
                                    y: "{l.y}",
                                    text_anchor: if l.end { "end" } else { "start" },
                                    dominant_baseline: "central",
                                    "{d.label}"
                                    tspan { "data-part": "leader-share", " {share(s.fraction)}" }
                                }
                            }
                        }
                    }
                }
                if labels == PieLabels::Inside {
                    for (i , s) in layout.slices.iter().enumerate().filter(|(_, s)| s.fraction >= pie::INSIDE_MIN) {
                        span {
                            "data-part": "slice-label",
                            "data-index": "{i}",
                            "aria-hidden": "true",
                            style: "left: {pct(s.centroid.0)}; top: {pct(s.centroid.1)}; --i: {i};",
                            "{share(s.fraction)}"
                        }
                    }
                }
                if let Some(i) = current {
                    div {
                        "data-part": "tooltip",
                        "data-side": if layout.slices[i].centroid.0 > 0.5 { "left" } else { "right" },
                        style: "left: {pct(layout.slices[i].centroid.0)}; top: {pct(layout.slices[i].centroid.1)};",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{parts[i].label}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {fmt(parts[i].value)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Share" }
                                span { "data-part": "tooltip-value", {share(layout.slices[i].fraction)} }
                            }
                        }
                    }
                }
            }
            if n > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , d) in parts.iter().enumerate() {
                        li {
                            "data-part": "legend-item",
                            "data-index": "{i}",
                            "data-active": (current == Some(i)).then_some(""),
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerleave: move |_| active.set(None),
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{d.label}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Part" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Share" }
                    }
                }
                tbody {
                    for (d , s) in parts.iter().zip(layout.slices.iter()) {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                            td { {share(s.fraction)} }
                        }
                    }
                }
            }
        }
    }
}
