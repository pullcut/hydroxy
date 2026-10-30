//! `<CartogramChart>`: regions resized by value, as a Dorling cartogram (one circle per
//! region) or a non-contiguous cartogram (each region shrunk about its centroid).
//! Geometry comes from `hydroxy_charts::cartogram`; no map data is bundled.

use std::fmt::Write as _;

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::{self, HINT_REGIONS, hinted};
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{cartogram, format_value};
use leptos::prelude::*;

pub use hydroxy_charts::cartogram::CartogramKind;
pub use hydroxy_charts::geo::{Projection, Region};

use crate::callbacks::{select_active, watch_hover};
use crate::geo_map::value_text;
use crate::plot_kit::{ChartTip, colour};

/// Fill and entrance for region `i` of `n`: its own chart colour, else the value shade.
pub(crate) fn mark_style(r: &Region, t: Option<f64>, i: usize, n: usize) -> String {
    let mut style = String::new();
    if let Some(fill) = r.colour {
        let _ = write!(style, "--fill: {};", colour(fill));
    }
    if let Some(t) = t {
        let _ = write!(style, "--t: {t:.3}; ");
    }
    let _ = write!(style, "--i: {i}; --ox-n: {n};");
    style
}

/// Regions resized so their area tracks their value. The default Dorling cartogram draws
/// each region as a labelled circle near where it sits on the map; the non-contiguous
/// kind keeps each region's shape and shrinks it about its centroid, so the densest
/// region keeps its size. A faint outline of the map stays underneath for reference.
/// Hover, touch or focus the map and use the arrow keys to read each region, Home and End
/// to jump, Escape to clear.
#[component]
pub fn CartogramChart(
    /// The regions, as rings of `(longitude, latitude)` degrees. Dorling circles are
    /// labelled with each region's id, so keep ids short (state or country codes).
    #[prop(into)]
    regions: Signal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Name of the measure, such as "Population (m)". Heads the table column.
    #[prop(into)]
    measure: String,
    /// Dorling circles (default) or non-contiguous shapes.
    #[prop(optional)]
    kind: CartogramKind,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[prop(optional)]
    projection: Projection,
    /// Formats values on the legend, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| regions.with(|r| cartogram::layout(r, projection)));
    let active = RwSignal::new(None::<usize>);
    let dorling = kind == CartogramKind::Dorling;
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            regions.with(|r| r.get(i).map(|r| EventDatum::category(i, r.name.clone(), r.value)))
        })
    });
    watch_hover(datum, on_hover);
    let text = move |i: usize| {
        regions.with(|r| r.get(i).map(|r| format!("{}: {}", r.name, value_text(r.value, fmt)))).unwrap_or_default()
    };
    let aria = hinted(&label, HINT_REGIONS);
    let measure_tip = StoredValue::new(measure.clone());

    view! {
        <figure
            data-scope="cartogram-chart"
            data-part="root"
            data-kind=if dorling { "dorling" } else { "non-contiguous" }
            data-active=move || active.get().map(|_| "")
        >
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=move || layout.with(|l| format!("0 0 {} {:.2}", l.geo.width, l.geo.height))
                    tabindex="0"
                    role="group"
                    aria-label=aria
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            select_active(datum, on_select);
                        }
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), regions.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    <g data-part="land" aria-hidden="true">
                        {move || layout.get().geo.shapes.into_iter().map(|s| view! { <path d=s.path></path> }).collect_view()}
                    </g>
                    {move || {
                        let l = layout.get();
                        let rs = regions.get();
                        // Biggest first, so small marks stay on top and reachable.
                        let n = rs.len().min(l.cells.len());
                        let mut order: Vec<usize> = (0..n).collect();
                        order.sort_by(|&a, &b| l.cells[b].r.total_cmp(&l.cells[a].r));
                        order
                            .into_iter()
                            .filter(|&i| if dorling { l.cells[i].r > 0.0 } else { l.cells[i].k > 0.0 })
                            .map(|i| {
                                let (region, cell, shape) = (&rs[i], l.cells[i], &l.geo.shapes[i]);
                                let style = mark_style(region, shape.t, i, n);
                                let on_enter = move |_| active.set(Some(i));
                                let on_click = move |_| {
                                    active.set(Some(i));
                                    select_active(datum, on_select);
                                };
                                let is_active = move || (active.get() == Some(i)).then_some("");
                                if dorling {
                                    let (dx, dy) = (cell.centroid.0 - cell.at.0, cell.centroid.1 - cell.at.1);
                                    let label = (cell.r >= 11.0).then(|| region.id.clone());
                                    view! {
                                        <g data-part="cell" data-active=is_active style=format!("{style} --dx: {dx:.2}px; --dy: {dy:.2}px;")
                                            on:pointerenter=on_enter on:pointerdown=on_enter on:click=on_click>
                                            <circle data-part="circle" cx=format!("{:.2}", cell.at.0) cy=format!("{:.2}", cell.at.1) r=format!("{:.2}", cell.r)></circle>
                                            {label.map(|t| view! {
                                                <text data-part="label" x=format!("{:.2}", cell.at.0) y=format!("{:.2}", cell.at.1)>{t}</text>
                                            })}
                                        </g>
                                    }
                                    .into_any()
                                } else {
                                    let origin = format!("transform-origin: {:.2}px {:.2}px; --k: {:.4};", cell.centroid.0, cell.centroid.1, cell.k);
                                    view! {
                                        <path data-part="region" data-active=is_active d=shape.path.clone() style=format!("{style} {origin}")
                                            on:pointerenter=on_enter on:pointerdown=on_enter on:click=on_click></path>
                                    }
                                    .into_any()
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let region = regions.with(|r| r.get(i).cloned())?;
                    let (x, y, fill, share) = layout.with(|l| {
                        let cell = l.cells.get(i)?;
                        let (x, y) = if dorling { (cell.at.0, cell.at.1 - cell.r) } else { cell.centroid };
                        let fill = colour(region.colour.unwrap_or(0));
                        Some((x / l.geo.width * 100.0, y / l.geo.height * 100.0, fill, cell.k))
                    })?;
                    let mut rows = vec![TipRow::new(None, measure_tip.get_value(), value_text(region.value, fmt))];
                    if !dorling && region.value.is_finite() {
                        rows.push(TipRow::new(None, "Drawn at", format!("{:.0}% of its area", share * share * 100.0)));
                    }
                    // The anchor only picks the side and vertical placement; `style` positions it.
                    let tip = Tip {
                        x: if x > 50.0 { 1.0 } else { 0.0 },
                        y: if y < 25.0 { 1.0 } else { 0.0 },
                        title: region.name,
                        rows,
                    };
                    Some(view! { <ChartTip tip swatch=fill style=format!("left: {x:.2}%; top: {y:.2}%;") /> })
                }}
            </div>
            <div data-part="legend" aria-hidden="true">
                <output data-part="readout">{move || active.get().map_or_else(|| "Hover or focus a region".to_string(), text)}</output>
                <Show when=move || regions.with(|r| r.iter().any(|r| r.colour.is_none() && r.value.is_finite()))>
                    <div data-part="scale">
                        <span>{move || fmt(layout.with(|l| l.geo.range.0))}</span>
                        <span data-part="ramp"></span>
                        <span>{move || fmt(layout.with(|l| l.geo.range.1))}</span>
                    </div>
                </Show>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(text).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr><th scope="col">"Region"</th><th scope="col">{measure}</th></tr>
                </thead>
                <tbody>
                    {move || {
                        regions
                            .get()
                            .into_iter()
                            .map(|r| view! { <tr><th scope="row">{r.name}</th><td>{value_text(r.value, fmt)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
