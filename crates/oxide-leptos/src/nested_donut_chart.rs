//! `<NestedDonutChart>`: concentric rings, innermost first. A ring is either one level of
//! a tree, with children under their parent's sweep, or a separate series over the same
//! categories. Geometry comes from `oxide_charts::nested_donut`.

use leptos::prelude::*;
use oxide_charts::{chord, format_value, nav, nested_donut};

use crate::sets_kit::SetsTip;

pub use oxide_charts::Datum;
pub use oxide_charts::hierarchy::Tree;
pub use oxide_charts::nested_donut::{Part, Ring, levels, series};

/// A nested donut. Build the rings with [`levels`] (one ring per tree level) or [`series`]
/// (one ring per series). The whole chart sweeps in clockwise as one motion. Hover or
/// touch a segment to read it; focus the chart and use the arrow keys to step through the
/// segments ring by ring, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn NestedDonutChart(
    /// The rings, innermost first.
    #[prop(into)]
    rings: Signal<Vec<Ring>>,
    /// What the chart shows, such as "Traffic by source, 2024 and 2025". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Diameter in px.
    #[prop(default = 280)]
    size: u16,
    /// The empty centre's share of the radius, 0 to 0.9.
    #[prop(default = 0.42)]
    hole: f64,
    /// Small caption under the centre figure. Default: the outer ring's name.
    #[prop(optional, into)]
    center_label: Option<String>,
    /// Formats values for the centre, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let s = f64::from(size);
    let layout = Memo::new(move |_| rings.with(|r| nested_donut::layout(r, s, hole, 3.0, 2.0)));
    let active = RwSignal::new(None::<usize>);
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let tip = move || active.get().and_then(|i| rings.with(|r| layout.with(|l| l.tip(r, i, &fmt))));
    let centre = move || {
        rings.with(|r| {
            layout.with(|l| {
                if let Some(seg) = active.get().and_then(|i| l.segments.get(i)) {
                    let part = &r[seg.ring].parts[seg.part];
                    (fmt(part.value), part.label.clone())
                } else {
                    let outer = l.rings.last().map_or(0.0, |x| x.total);
                    (fmt(outer), center_label.clone().unwrap_or_else(|| r.last().map(|x| x.name.clone()).unwrap_or_default()))
                }
            })
        })
    };
    let group_label = format!("{label}. Use the arrow keys to step through the segments.");
    let caption = label;

    view! {
        <figure data-scope="nested-donut-chart" data-part="root" data-active=move || active.get().map(|_| "")>
            <div
                data-part="plot"
                style=format!("width: {size}px; height: {size}px;")
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), layout.with(|l| l.segments.len())) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {size} {size}") width=size height=size aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        let r = rings.get();
                        l.rings
                            .iter()
                            .enumerate()
                            .map(|(k, ring)| {
                                let segs = l
                                    .segments
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, seg)| seg.ring == k)
                                    .map(|(i, seg)| {
                                        view! {
                                            <path
                                                data-part="segment"
                                                data-active=move || (active.get() == Some(i)).then_some("")
                                                d=seg.path.clone()
                                                stroke=l.colour(&r, i)
                                                stroke-width=ring.thickness
                                                stroke-opacity=r[k].opacity
                                                on:pointerenter=move |_| pick(i)
                                                on:pointerdown=move |_| pick(i)
                                            ></path>
                                        }
                                    })
                                    .collect_view();
                                view! {
                                    <g data-part="ring" data-ring=k transform=format!("translate({:.3} {:.3})", ring.offset, ring.offset)>
                                        {segs}
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                <div data-part="center" aria-hidden="true" style=move || format!("width: {:.1}px;", layout.with(|l| l.hole) * 2.0 - 8.0)>
                    {move || {
                        let (value, name) = centre();
                        view! {
                            <span data-part="center-value">{value}</span>
                            <span data-part="center-label">{name}</span>
                        }
                    }}
                </div>
                {move || tip().map(|tip| view! { <SetsTip tip=tip /> })}
            </div>
            <div data-part="key" aria-hidden="true">
                <ul data-part="legend">
                    {move || {
                        let names = rings.with(|r| nested_donut::legend(r));
                        let n = names.len();
                        let mut items: Vec<(String, String)> = names
                            .into_iter()
                            .enumerate()
                            .filter(|(i, _)| chord::slot(*i, n).is_some())
                            .map(|(i, t)| (chord::colour(i, n), t))
                            .collect();
                        if n > 6 {
                            items.push(("var(--chart-muted)".to_string(), "Other".to_string()));
                        }
                        items
                            .into_iter()
                            .map(|(c, t)| {
                                view! {
                                    <li data-part="legend-item">
                                        <span data-part="swatch" style=format!("background: {c};")></span>
                                        {t}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
                {move || {
                    let names: Vec<String> = rings.with(|r| r.iter().map(|x| x.name.clone()).collect());
                    (names.len() > 1).then(|| view! { <p data-part="ring-key">{format!("Rings from the centre out: {}", names.join(", "))}</p> })
                }}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || tip().map(|t| t.announce()).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Ring"</th>
                        <th scope="col">"Part"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Share"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let r = rings.get();
                        layout
                            .get()
                            .segments
                            .into_iter()
                            .map(|seg| {
                                let ring = &r[seg.ring];
                                let part = &ring.parts[seg.part];
                                view! {
                                    <tr>
                                        <td>{ring.name.clone()}</td>
                                        <th scope="row">{part.label.clone()}</th>
                                        <td>{fmt(part.value)}</td>
                                        <td>{oxide_charts::upset::percent(seg.fraction, 1.0)}</td>
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
