//! `<DonutChart>`: parts of a whole as a ring with a legend. Geometry comes from
//! `hydroxy_charts::donut`. More than six parts fold into "Other".

use hydroxy_charts::{donut, fold_other, format_value, nav};
use leptos::prelude::*;

pub use hydroxy_charts::Datum;

/// Colour for slot `i`: the categorical order, with a folded "Other" in neutral grey.
fn slot_colour(i: usize, folded_other: bool, len: usize) -> String {
    if folded_other && i + 1 == len { "var(--chart-muted)".into() } else { format!("var(--chart-{})", i % 6 + 1) }
}

/// A ring chart with a total in the middle and a legend with values and shares.
/// Hover or touch a segment or legend row to read it; focus the ring and use the arrow
/// keys to step through segments, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
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
    /// Formats values for the centre, legend, tooltip and table.
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
    let active = RwSignal::new(None::<usize>);
    // Segments are keyed by label: a changed share reshapes its own arc; new parts sweep in.
    let rows = move || parts.get().into_iter().enumerate().collect::<Vec<_>>();
    let total = Memo::new(move |_| parts.with(|p| p.iter().map(|d| d.value.max(0.0)).sum::<f64>()));
    let centre = move || center_value.get().unwrap_or_else(|| fmt(total.get()));
    let share = move |i: usize| segs.with(|s| s.get(i).map_or(0.0, |x| x.fraction));
    let describe = move |i: usize| {
        parts.with(|p| p.get(i).map(|d| format!("{}: {}, {:.0}%", d.label, fmt(d.value), share(i) * 100.0)))
    };
    let set_active = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };

    let group_label = format!("{label}. Use the arrow keys to step through segments.");
    let caption = label.clone();

    view! {
        <figure
            data-scope="donut-chart"
            data-part="root"
            aria-label=label
            data-active=move || active.get().map(|_| "")
            style=move || format!("--ox-segments: {};", parts.with(Vec::len))
        >
            <div
                data-part="ring"
                style=format!("width: {size}px; height: {size}px;")
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), parts.with(Vec::len)) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {size} {size}") width=size height=size aria-hidden="true">
                    <For
                        each=rows
                        key=|(_, d)| d.label.clone()
                        children=move |(i, d)| {
                            let label = d.label;
                            let pos = Memo::new(move |_| parts.with(|p| p.iter().position(|x| x.label == label)));
                            let path = move || pos.get().and_then(|p| segs.with(|s| s.get(p).map(|x| x.path.clone()))).unwrap_or_default();
                            let colour = move || slot_colour(pos.get().unwrap_or(i), folded.get(), parts.with(Vec::len));
                            let pick = move |_| set_active(pos.get_untracked().unwrap_or(i));
                            view! {
                                <path
                                    data-part="segment"
                                    data-index=move || pos.get().unwrap_or(i)
                                    data-active=move || (active.get().is_some() && active.get() == pos.get()).then_some("")
                                    style=format!("--i: {i};")
                                    d=path
                                    stroke=colour
                                    stroke-width=ring.stroke_width
                                    pathLength="1"
                                    on:pointerenter=pick
                                    on:pointerdown=pick
                                ></path>
                            }
                        }
                    />
                </svg>
                <div data-part="center" aria-hidden="true">
                    <span data-part="center-value">{centre}</span>
                    {center_label.map(|l| view! { <span data-part="center-label">{l}</span> })}
                </div>
                {move || {
                    let i = active.get()?;
                    let part = parts.with(|p| p.get(i).cloned())?;
                    let values: Vec<f64> = parts.with(|p| p.iter().map(|d| d.value).collect());
                    let (x, y) = donut::anchor(&values, i, f64::from(size), f64::from(thickness))?;
                    let n = parts.with(Vec::len);
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if x > 0.5 { "right" } else { "left" }}
                                style=format!("left: {:.3}%; top: {:.3}%;", x * 100.0, y * 100.0)
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: {};", slot_colour(i, folded.get(), n))></span>
                                    {part.label}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Value"</span>
                                        <span data-part="tooltip-value">{fmt(part.value)}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Share"</span>
                                        <span data-part="tooltip-value">{format!("{:.0}%", share(i) * 100.0)}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    let p = parts.get();
                    let n = p.len();
                    let segs = segs.get();
                    p.into_iter()
                        .zip(segs)
                        .enumerate()
                        .map(|(i, (d, s))| {
                            view! {
                                <li
                                    data-part="legend-item"
                                    data-index=i
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    on:pointerenter=move |_| set_active(i)
                                    on:pointerleave=move |_| active.set(None)
                                >
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
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Part"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Share"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        parts
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, d)| {
                                view! {
                                    <tr>
                                        <th scope="row">{d.label}</th>
                                        <td>{fmt(d.value)}</td>
                                        <td>{format!("{:.0}%", share(i) * 100.0)}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
