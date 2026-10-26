//! `<Sparkline>`: a tiny inline trend line with no axes. Geometry comes from
//! `oxide_charts::sparkline`.

use leptos::prelude::*;
use oxide_charts::{format_value, sparkline};

pub use oxide_charts::line::Curve;

/// Room around the line for the stroke and the last-point dot.
const PAD: f64 = 3.0;

/// A small line showing the shape of a series, sized in px. Its accessible name
/// summarises the low, high and latest values.
#[component]
pub fn Sparkline(
    /// The values, oldest first. Non-finite values leave gaps.
    #[prop(into)]
    values: Signal<Vec<f64>>,
    /// What the line shows, such as "Signups, last 30 days". Starts the accessible name.
    #[prop(into)]
    label: String,
    /// Width in px.
    #[prop(default = 96)]
    width: u16,
    /// Height in px.
    #[prop(default = 24)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[prop(optional)]
    curve: Curve,
    /// Fill the area under the line.
    #[prop(optional)]
    area: bool,
    /// Mark the latest value with a dot.
    #[prop(default = true)]
    dot: bool,
    /// Formats values in the accessible summary.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| values.with(|v| sparkline::layout(v, f64::from(width), f64::from(height), PAD, curve)));
    let summary = move || {
        layout.with(|l| match (l.min, l.max) {
            (Some(lo), Some(hi)) => {
                let latest = l.latest.map_or_else(|| "missing".to_string(), fmt);
                format!("{label}: low {}, high {}, latest {latest}", fmt(lo), fmt(hi))
            }
            _ => format!("{label}: no data"),
        })
    };

    view! {
        <svg
            data-scope="sparkline"
            data-part="root"
            role="img"
            aria-label=summary
            width=width
            height=height
            viewBox=format!("0 0 {width} {height}")
        >
            {move || area.then(|| view! { <path data-part="area" d=layout.with(|l| l.area.clone())></path> })}
            <path data-part="line" d=move || layout.with(|l| l.line.clone())></path>
            {move || {
                layout.with(|l| l.last).filter(|_| dot).map(|(cx, cy)| view! { <circle data-part="dot" cx=cx cy=cy r="2"></circle> })
            }}
        </svg>
    }
}
