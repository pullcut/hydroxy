//! `<BubbleMapChart>`: proportional circles at longitude and latitude over base map
//! shapes. Geometry comes from `hydroxy_charts::geo`; no map data is bundled.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::{self, HINT_POINTS, hinted};
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, geo};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Circles whose area is their value, drawn over plain regions. Hover, touch or focus the
/// map and use the arrow keys to step through the points in the order given, Home and End
/// to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn BubbleMapChart(
    /// The points. Give each a group (0 to 5) to colour it by series.
    #[prop(into)]
    points: Signal<Vec<GeoPoint>>,
    /// Base shapes drawn under the bubbles, as rings of `(longitude, latitude)`. Their values are ignored.
    #[prop(optional, into)]
    regions: Signal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Name of the measure, such as "Orders". Heads the table column.
    #[prop(into)]
    measure: String,
    /// Series names for the legend, by group index. The legend shows when there are two or more.
    #[prop(optional, into)]
    groups: Signal<Vec<String>>,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[prop(optional)]
    projection: Projection,
    /// Radius of the largest bubble, in viewBox units (the map is 640 wide).
    #[prop(default = 28.0)]
    max_radius: f64,
    /// Formats values on the tooltip, legend and table. Default: whole numbers, else one decimal place.
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
    let layout = Memo::new(move |_| regions.with(|r| points.with(|p| geo::layout(r, p, projection))));
    let max =
        Memo::new(move |_| points.with(|p| p.iter().map(|p| p.value).filter(|v| v.is_finite()).fold(0.0, f64::max)));
    let active = RwSignal::new(None::<usize>);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            points.with(|p| {
                let p = p.get(i)?;
                Some(EventDatum { series: p.group, ..EventDatum::category(i, p.name.clone(), p.value) })
            })
        })
    });
    watch_hover(datum, on_hover);
    let text = move |i: usize| {
        points
            .with(|p| {
                p.get(i).map(|p| {
                    format!("{}: {}", p.name, if p.value.is_finite() { fmt(p.value) } else { "No data".into() })
                })
            })
            .unwrap_or_default()
    };
    let aria = hinted(&label, HINT_POINTS);
    let measure_tip = measure.clone();

    view! {
        <figure data-scope="bubble-map-chart" data-part="root" data-active=move || active.get().map(|_| "")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=move || layout.with(|l| format!("0 0 {} {:.2}", l.width, l.height))
                    tabindex="0"
                    role="group"
                    aria-label=aria
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            select_active(datum, on_select);
                        }
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), points.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    <g data-part="land" aria-hidden="true">
                        {move || layout.get().shapes.into_iter().map(|s| view! { <path d=s.path></path> }).collect_view()}
                    </g>
                    {move || {
                        let l = layout.get();
                        let max = max.get();
                        let pts = points.get();
                        geo::draw_order(&pts)
                            .into_iter()
                            .filter_map(|i| l.points.get(i).copied().flatten().map(|at| (i, at)))
                            .map(|(i, (x, y))| {
                                let p = &pts[i];
                                view! {
                                    <circle
                                        data-part="bubble"
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        cx=format!("{x:.2}")
                                        cy=format!("{y:.2}")
                                        r=format!("{:.2}", geo::bubble_radius(p.value, max, max_radius).max(2.0))
                                        style=format!("fill: {}; --i: {};", colour(p.group), i.min(40))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                        on:click=move |_| {
                                            active.set(Some(i));
                                            select_active(datum, on_select);
                                        }
                                    ></circle>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let p = points.with(|p| p.get(i).cloned())?;
                    let (left, top) = layout.with(|l| l.points.get(i).copied().flatten().map(|(x, y)| (x / l.width * 100.0, y / l.height * 100.0)))?;
                    let group = groups.with(|g| g.get(p.group).cloned());
                    let mut rows = vec![TipRow::new(
                        None,
                        measure_tip.clone(),
                        if p.value.is_finite() { fmt(p.value) } else { "No data".into() },
                    )];
                    rows.extend(group.map(|g| TipRow::new(None, "Series", g)));
                    // The anchor only picks the side and vertical placement; `style` positions it.
                    let tip = Tip {
                        x: if left > 50.0 { 1.0 } else { 0.0 },
                        y: if top < 25.0 { 1.0 } else { 0.0 },
                        title: p.name,
                        rows,
                    };
                    Some(view! { <ChartTip tip swatch=colour(p.group) style=format!("left: {left:.2}%; top: {top:.2}%;") /> })
                }}
            </div>
            <Show when=move || groups.with(|g| g.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        groups
                            .get()
                            .into_iter()
                            .take(6)
                            .enumerate()
                            .map(|(i, g)| view! { <li data-part="legend-item"><span data-part="swatch" style=format!("background: {};", colour(i))></span>{g}</li> })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(text).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Name"</th>
                        <th scope="col">"Longitude"</th>
                        <th scope="col">"Latitude"</th>
                        <th scope="col">{measure}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        points
                            .get()
                            .into_iter()
                            .map(|p| view! {
                                <tr>
                                    <th scope="row">{p.name}</th>
                                    <td>{format!("{:.2}", p.lon)}</td>
                                    <td>{format!("{:.2}", p.lat)}</td>
                                    <td>{if p.value.is_finite() { fmt(p.value) } else { "–".into() }}</td>
                                </tr>
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
