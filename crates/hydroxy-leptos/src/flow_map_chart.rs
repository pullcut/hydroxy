//! `<FlowMapChart>`: curved arrows from origin to destination over base map shapes, as
//! wide as their volume. Geometry comes from `hydroxy_charts::flow_map`; no map data is bundled.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{flow_map, format_value, nav};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::flow_map::Flow;
pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Movements between places, each a curved arrow from its origin to its destination whose
/// width is its volume. Every arrow bows to the same side of its direction, so a flow and
/// its return run side by side. Hover, touch or focus the map and use the arrow keys to
/// step through the flows in the order given, Home and End to jump, Escape to clear; the
/// picked flow and its two ends stand out.
#[component]
pub fn FlowMapChart(
    /// The places flows run between. Only their name and position are used.
    #[prop(into)]
    places: Signal<Vec<GeoPoint>>,
    /// The flows: `Flow::new(from, to, volume)` with indices into `places`.
    #[prop(into)]
    flows: Signal<Vec<Flow>>,
    /// Base shapes drawn under the flows, as rings of `(longitude, latitude)`. Their values are ignored.
    #[prop(optional, into)]
    regions: Signal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Name of the volume, such as "Passengers". Heads the table column.
    #[prop(into)]
    measure: String,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[prop(optional)]
    projection: Projection,
    /// Width of the biggest flow, in viewBox units (the map is 640 wide).
    #[prop(default = 12.0)]
    max_width: f64,
    /// Formats volumes on the tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a flow, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a flow.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let value_text = move |v: f64| if v.is_finite() { fmt(v) } else { "No data".to_string() };
    let layout = Memo::new(move |_| {
        places.with(|p| flows.with(|f| regions.with(|r| flow_map::layout(p, f, r, projection, max_width))))
    });
    let active = RwSignal::new(None::<usize>);
    let name = move |i: usize| places.with(|p| p.get(i).map(|p| p.name.clone())).unwrap_or_default();
    let ends = move || active.get().and_then(|i| flows.with(|f| f.get(i).map(|f| (f.from, f.to))));
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let (from, to, value) = flows.with(|f| f.get(i).map(|f| (f.from, f.to, f.value)))?;
        Some(EventDatum {
            series: 0,
            index: i,
            label: Some(format!("{} to {}", name(from), name(to))),
            value,
            node: None,
        })
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let text = move |i: usize| {
        flows
            .with(|f| f.get(i).map(|f| format!("{} to {}: {}", name(f.from), name(f.to), value_text(f.value))))
            .unwrap_or_default()
    };
    let aria = format!("{label}. Use the arrow keys to step through the flows.");
    let measure_tip = StoredValue::new(measure.clone());

    view! {
        <figure data-scope="flow-map-chart" data-part="root" data-active=move || active.get().map(|_| "")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=move || layout.with(|l| format!("0 0 {} {:.2}", l.geo.width, l.geo.height))
                    tabindex="0"
                    role="group"
                    aria-label=aria
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), flows.with(Vec::len)) {
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
                        let flows = l.order.iter().filter(|&&i| l.arcs[i].is_some()).count();
                        l.order
                            .iter()
                            .filter_map(|&i| l.arcs[i].clone().map(|a| (i, a)))
                            .enumerate()
                            .map(|(k, (i, a))| {
                                // Widest first in paint order, so the widest flow also draws first.
                                let on_enter = move |_| active.set(Some(i));
                                view! {
                                    <g
                                        data-part="flow"
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {k}; --ox-n: {flows};")
                                        on:pointerenter=on_enter
                                        on:pointerdown=on_enter
                                    >
                                        <path data-part="hit" d=a.path.clone() stroke-width=format!("{:.2}", a.width + 10.0)></path>
                                        <path data-part="line" d=a.path pathLength="1" stroke-width=format!("{:.2}", a.width)></path>
                                        <path data-part="head" d=a.head></path>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let l = layout.get();
                        let places = places.get();
                        let n = places.len();
                        places
                            .into_iter()
                            .zip(l.geo.points)
                            .enumerate()
                            .filter_map(|(k, (p, at))| at.map(|at| (k, p, at)))
                            .map(|(k, p, (x, y))| {
                                let end = move || ends().and_then(|(a, b)| if a == k { Some("origin") } else if b == k { Some("destination") } else { None });
                                let anchor = if x > l.geo.width * 0.8 { "end" } else { "start" };
                                let tx = if anchor == "end" { x - 8.0 } else { x + 8.0 };
                                view! {
                                    <g data-part="place" data-end=end style=format!("--i: {k}; --ox-n: {n};")>
                                        <circle cx=format!("{x:.2}") cy=format!("{y:.2}") r=flow_map::PLACE_RADIUS_ATTR></circle>
                                        <text data-part="label" x=format!("{tx:.2}") y=format!("{y:.2}") text-anchor=anchor>{p.name}</text>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let f = flows.with(|f| f.get(i).cloned())?;
                    let (left, top) = layout.with(|l| l.arcs.get(i).cloned().flatten().map(|a| (a.mid.0 / l.geo.width * 100.0, a.mid.1 / l.geo.height * 100.0)))?;
                    // The tooltip opens below the arrow near the top edge (top < 25%), above it elsewhere.
                    let tip = Tip {
                        x: left / 100.0,
                        y: if top < 25.0 { 1.0 } else { 0.0 },
                        title: format!("{} → {}", name(f.from), name(f.to)),
                        rows: vec![TipRow::new(None, measure_tip.get_value(), value_text(f.value))],
                    };
                    let style = format!("left: {left:.2}%; top: {top:.2}%;");
                    Some(view! { <ChartTip tip swatch=colour(0) style /> })
                }}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(text).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"From"</th>
                        <th scope="col">"To"</th>
                        <th scope="col">{measure}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        flows
                            .get()
                            .into_iter()
                            .map(|f| view! {
                                <tr>
                                    <th scope="row">{name(f.from)}</th>
                                    <td>{name(f.to)}</td>
                                    <td>{value_text(f.value)}</td>
                                </tr>
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
