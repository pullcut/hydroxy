//! `Sparkline`: a tiny inline trend line with no axes. Geometry comes from
//! `hydroxy_charts::sparkline`.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, nav, sparkline};

pub use hydroxy_charts::line::Curve;

/// Room around the line for the stroke and the last-point dot.
const PAD: f64 = 3.0;

/// A small line showing the shape of a series, sized in px. Its accessible name
/// summarises the low, high and latest values. Hover or touch to read the nearest
/// point; focus it and use the arrow keys to step through points, Home and End to
/// jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn Sparkline(
    /// The values, oldest first. Non-finite values leave gaps.
    values: ReadSignal<Vec<f64>>,
    /// What the line shows, such as "Signups, last 30 days". Starts the accessible name.
    label: String,
    /// Width in px.
    #[props(default = 96)]
    width: u16,
    /// Height in px.
    #[props(default = 24)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[props(default)]
    curve: Curve,
    /// Fill the area under the line.
    #[props(default)]
    area: bool,
    /// Mark the latest value with a dot.
    #[props(default = true)]
    dot: bool,
    /// Formats values in the accessible summary, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let show = move |v: Option<f64>| v.filter(|v| v.is_finite()).map_or_else(|| "missing".to_string(), fmt);
    let vals = values();
    let n = vals.len();
    let l = sparkline::layout(&vals, f64::from(width), f64::from(height), PAD, curve);
    let summary = match (l.min, l.max) {
        (Some(lo), Some(hi)) => format!("{label}: low {}, high {}, latest {}. Use the arrow keys to read each value.", fmt(lo), fmt(hi), show(l.latest)),
        _ => format!("{label}: no data"),
    };
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let pick = move |e: Event<PointerData>| {
        let next = sparkline::index_at(e.data().element_coordinates().x, n, f64::from(width), PAD);
        if active() != next {
            active.set(next);
        }
    };
    let tip = current.map(|i| (i, l.points[i].unwrap_or((f64::from(width) / 2.0, f64::from(height) / 2.0))));
    let active_dot = current.and_then(|i| l.points[i]);
    let announcement = current.map(|i| format!("{} of {n}: {}", i + 1, show(vals.get(i).copied())));

    rsx! {
        span {
            "data-scope": "sparkline",
            "data-part": "root",
            tabindex: "0",
            role: "group",
            "aria-label": "{summary}",
            "data-active": current.map(|_| ""),
            style: "width: {width}px; height: {height}px;",
            onpointermove: pick,
            onpointerdown: pick,
            onpointerleave: move |_| active.set(None),
            onblur: move |_| active.set(None),
            onkeydown: move |e| {
                if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                    e.prevent_default();
                    active.set(next);
                }
            },
            svg { width: "{width}", height: "{height}", view_box: "0 0 {width} {height}", "aria-hidden": "true",
                if area {
                    path { "data-part": "area", d: "{l.area}" }
                }
                path { "data-part": "line", d: "{l.line}" }
                if let Some((cx, cy)) = l.last.filter(|_| dot) {
                    circle { "data-part": "dot", cx: "{cx}", cy: "{cy}", r: "2" }
                }
                if let Some((cx, cy)) = active_dot {
                    circle { "data-part": "active-dot", cx: "{cx}", cy: "{cy}", r: "3" }
                }
            }
            if let Some((i, (x, y))) = tip {
                span {
                    "data-part": "tooltip",
                    "data-anchored": "",
                    "data-side": if x > f64::from(width) / 2.0 { "left" } else { "right" },
                    style: "left: {x:.1}px; top: {y:.1}px;",
                    "aria-hidden": "true",
                    span { "data-part": "tooltip-label",
                        span { "data-part": "swatch", style: "background: var(--sparkline-color, var(--chart-1));" }
                        "{label}"
                    }
                    span { "data-part": "tooltip-row",
                        span { "data-part": "tooltip-name", "{i + 1} of {n}" }
                        span { "data-part": "tooltip-value", {show(vals.get(i).copied())} }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            // An ARIA table of spans: a sparkline often sits inside a paragraph, where <table> can't.
            span { "data-part": "sr-table", role: "table", "aria-label": "{label}",
                span { role: "row",
                    span { role: "columnheader", "Point" }
                    span { role: "columnheader", "Value" }
                }
                for (i, v) in vals.iter().enumerate() {
                    span { role: "row",
                        span { role: "rowheader", "{i + 1}" }
                        span { role: "cell", {show(Some(*v))} }
                    }
                }
            }
        }
    }
}
