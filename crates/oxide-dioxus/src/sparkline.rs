//! `Sparkline`: a tiny inline trend line with no axes. Geometry comes from
//! `oxide_charts::sparkline`.

use dioxus::prelude::*;
use oxide_charts::{format_value, sparkline};

pub use oxide_charts::line::Curve;

/// Room around the line for the stroke and the last-point dot.
const PAD: f64 = 3.0;

/// A small line showing the shape of a series, sized in px. Its accessible name
/// summarises the low, high and latest values.
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
    /// Formats values in the accessible summary.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let l = sparkline::layout(&values(), f64::from(width), f64::from(height), PAD, curve);
    let summary = match (l.min, l.max) {
        (Some(lo), Some(hi)) => {
            let latest = l.latest.map_or_else(|| "missing".to_string(), fmt);
            format!("{label}: low {}, high {}, latest {latest}", fmt(lo), fmt(hi))
        }
        _ => format!("{label}: no data"),
    };

    rsx! {
        svg {
            "data-scope": "sparkline",
            "data-part": "root",
            role: "img",
            "aria-label": "{summary}",
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            if area {
                path { "data-part": "area", d: "{l.area}" }
            }
            path { "data-part": "line", d: "{l.line}" }
            if let Some((cx, cy)) = l.last.filter(|_| dot) {
                circle { "data-part": "dot", cx: "{cx}", cy: "{cy}", r: "2" }
            }
        }
    }
}
