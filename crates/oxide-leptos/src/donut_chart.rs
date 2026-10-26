//! `<DonutChart>`: parts of a whole as a ring with a legend. Geometry comes from
//! `oxide_charts::donut`. More than six parts fold into "Other".

use leptos::prelude::*;
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
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Tickets by channel". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Ring diameter in px.
    #[prop(default = 180)]
    size: u16,
    /// Ring width in px.
    #[prop(default = 22)]
    thickness: u16,
    /// Small caption under the centre figure, such as "Tickets".
    #[prop(optional, into)]
    center_label: Option<String>,
    /// Centre figure. Default: the formatted total.
    #[prop(optional, into)]
    center_value: MaybeProp<String>,
    /// Formats values for the centre, legend and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let parts = Memo::new(move |_| data.with(|d| fold_other(d, 6)));
    let folded = Memo::new(move |_| data.with(Vec::len) > 6);
    let segs = Memo::new(move |_| {
        parts.with(|p| {
            let values: Vec<f64> = p.iter().map(|d| d.value).collect();
            donut::segments(&values, f64::from(size), f64::from(thickness), 2.0)
        })
    });
    let ring = donut::ring(f64::from(size), f64::from(thickness));
    // Segments are keyed by label: a changed share reshapes its own arc; new parts sweep in.
    let rows = move || parts.get().into_iter().enumerate().collect::<Vec<_>>();
    let total = Memo::new(move |_| parts.with(|p| p.iter().map(|d| d.value.max(0.0)).sum::<f64>()));
    let centre = move || center_value.get().unwrap_or_else(|| fmt(total.get()));

    view! {
        <figure
            data-scope="donut-chart"
            data-part="root"
            aria-label=label
            style=move || format!("--ox-segments: {};", parts.with(Vec::len))
        >
            <div data-part="ring" style=format!("width: {size}px; height: {size}px;") aria-hidden="true">
                <svg viewBox=format!("0 0 {size} {size}") width=size height=size>
                    <For
                        each=rows
                        key=|(_, d)| d.label.clone()
                        children=move |(i, d)| {
                            let label = d.label;
                            let pos = Memo::new(move |_| parts.with(|p| p.iter().position(|x| x.label == label)));
                            let path = move || pos.get().and_then(|p| segs.with(|s| s.get(p).map(|x| x.path.clone()))).unwrap_or_default();
                            let colour = move || slot_colour(pos.get().unwrap_or(i), folded.get(), parts.with(Vec::len));
                            view! {
                                <path
                                    data-part="segment"
                                    data-index=move || pos.get().unwrap_or(i)
                                    style=format!("--i: {i};")
                                    d=path
                                    stroke=colour
                                    stroke-width=ring.stroke_width
                                    pathLength="1"
                                ></path>
                            }
                        }
                    />
                </svg>
                <div data-part="center">
                    <span data-part="center-value">{centre}</span>
                    {center_label.map(|l| view! { <span data-part="center-label">{l}</span> })}
                </div>
            </div>
            <ul data-part="legend">
                {move || {
                    let p = parts.get();
                    let n = p.len();
                    let segs = segs.get();
                    p.into_iter()
                        .zip(segs)
                        .enumerate()
                        .map(|(i, (d, s))| {
                            view! {
                                <li data-part="legend-item" data-index=i>
                                    <span data-part="swatch" style=format!("background: {};", slot_colour(i, folded.get(), n))></span>
                                    <span data-part="legend-label">{d.label}</span>
                                    <span data-part="legend-value">{fmt(d.value)}</span>
                                    <span data-part="legend-share">{format!("{:.0}%", s.fraction * 100.0)}</span>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
        </figure>
    }
}
