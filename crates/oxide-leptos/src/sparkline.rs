//! `<Sparkline>`: a tiny inline trend line with no axes. Geometry comes from
//! `oxide_charts::sparkline`.

use leptos::prelude::*;
use oxide_charts::{format_value, nav, sparkline};

pub use oxide_charts::line::Curve;

/// Room around the line for the stroke and the last-point dot.
const PAD: f64 = 3.0;

/// A small line showing the shape of a series, sized in px. Its accessible name
/// summarises the low, high and latest values. Hover or touch to read the nearest
/// point; focus it and use the arrow keys to step through points, Home and End to
/// jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
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
    /// Formats values in the accessible summary, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let show = move |v: Option<f64>| v.filter(|v| v.is_finite()).map_or_else(|| "missing".to_string(), fmt);
    let layout = Memo::new(move |_| values.with(|v| sparkline::layout(v, f64::from(width), f64::from(height), PAD, curve)));
    let active = RwSignal::new(None::<usize>);
    let name = StoredValue::new(label);
    let summary = move || {
        let label = name.get_value();
        layout.with(|l| match (l.min, l.max) {
            (Some(lo), Some(hi)) => format!("{label}: low {}, high {}, latest {}. Use the arrow keys to read each value.", fmt(lo), fmt(hi), show(l.latest)),
            _ => format!("{label}: no data"),
        })
    };
    let value_at = move |i: usize| values.with(|v| v.get(i).copied());
    let set = move |i: Option<usize>| {
        if active.get_untracked() != i {
            active.set(i);
        }
    };
    let pick = move |ev: leptos::ev::PointerEvent| {
        set(sparkline::index_at(f64::from(ev.offset_x()), values.with(Vec::len), f64::from(width), PAD));
    };

    view! {
        <span
            data-scope="sparkline"
            data-part="root"
            tabindex="0"
            role="group"
            aria-label=summary
            data-active=move || active.get().map(|_| "")
            style=format!("width: {width}px; height: {height}px;")
            on:pointermove=pick
            on:pointerdown=pick
            on:pointerleave=move |_| active.set(None)
            on:blur=move |_| active.set(None)
            on:keydown=move |ev| {
                if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), values.with(Vec::len)) {
                    ev.prevent_default();
                    active.set(next);
                }
            }
        >
            <svg width=width height=height viewBox=format!("0 0 {width} {height}") aria-hidden="true">
                {move || area.then(|| view! { <path data-part="area" d=layout.with(|l| l.area.clone())></path> })}
                <path data-part="line" d=move || layout.with(|l| l.line.clone())></path>
                {move || {
                    layout.with(|l| l.last).filter(|_| dot).map(|(cx, cy)| view! { <circle data-part="dot" cx=cx cy=cy r="2"></circle> })
                }}
                {move || {
                    let i = active.get()?;
                    let (cx, cy) = layout.with(|l| l.points.get(i).copied().flatten())?;
                    Some(view! { <circle data-part="active-dot" cx=cx cy=cy r="3"></circle> })
                }}
            </svg>
            {move || {
                let i = active.get()?;
                let n = values.with(Vec::len);
                let (x, y) = layout.with(|l| l.points.get(i).copied().flatten()).unwrap_or((f64::from(width) / 2.0, f64::from(height) / 2.0));
                Some(
                    view! {
                        <span
                            data-part="tooltip"
                            data-anchored=""
                            data-side={if x > f64::from(width) / 2.0 { "left" } else { "right" }}
                            style=format!("left: {x:.1}px; top: {y:.1}px;")
                            aria-hidden="true"
                        >
                            <span data-part="tooltip-label">
                                <span data-part="swatch" style="background: var(--sparkline-color, var(--chart-1));"></span>
                                {name.get_value()}
                            </span>
                            <span data-part="tooltip-row">
                                <span data-part="tooltip-name">{format!("{} of {n}", i + 1)}</span>
                                <span data-part="tooltip-value">{show(value_at(i))}</span>
                            </span>
                        </span>
                    },
                )
            }}
            <span data-part="sr-live" aria-live="polite">
                {move || active.get().map(|i| format!("{} of {}: {}", i + 1, values.with(Vec::len), show(value_at(i)))).unwrap_or_default()}
            </span>
            // An ARIA table of spans: a sparkline often sits inside a paragraph, where <table> can't.
            <span data-part="sr-table" role="table" aria-label=name.get_value()>
                <span role="row">
                    <span role="columnheader">"Point"</span>
                    <span role="columnheader">"Value"</span>
                </span>
                {move || {
                    values
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| view! { <span role="row"><span role="rowheader">{i + 1}</span><span role="cell">{show(Some(v))}</span></span> })
                        .collect_view()
                }}
            </span>
        </span>
    }
}
