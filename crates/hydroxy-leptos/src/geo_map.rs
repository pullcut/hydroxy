//! `<GeoMap>`: a choropleth or polygon map over caller-supplied polygons, with optional
//! points on top. Geometry comes from `hydroxy_charts::geo`; no map data is bundled.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, geo, nav};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Value text, or "No data" for a missing value.
pub(crate) fn value_text(v: f64, fmt: impl Fn(f64) -> String) -> String {
    if v.is_finite() { fmt(v) } else { "No data".to_string() }
}

/// Name, row label, value, colour and anchor in viewBox units for one mark.
type Mark = (String, String, f64, String, (f64, f64));

/// Regions shaded on a sequential scale by value, or filled with their own colour when
/// they have one (a polygon map), with optional points on top. Hover, touch or focus the
/// map and use the arrow keys to read regions and then points, Home and End to jump,
/// Escape to clear.
#[component]
pub fn GeoMap(
    /// The regions, as rings of `(longitude, latitude)` degrees.
    #[prop(into)]
    regions: Signal<Vec<Region>>,
    /// Points drawn over the regions, such as offices or stores.
    #[prop(optional, into)]
    points: Signal<Vec<GeoPoint>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Name of the measure, such as "Customers per 1k people". Heads the table column.
    #[prop(into)]
    measure: String,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[prop(optional)]
    projection: Projection,
    /// Formats values on the legend, tooltip, readout and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a region or point, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a region or point.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| regions.with(|r| points.with(|p| geo::layout(r, p, projection))));
    let active = RwSignal::new(None::<usize>);
    let counts = move || (regions.with(Vec::len), points.with(Vec::len));
    // (name, row label, value, colour, anchor in viewBox units) for mark `i`.
    let measure_tip = StoredValue::new(measure.clone());
    let mark = move |i: usize| -> Option<Mark> {
        let (n, _) = counts();
        if i < n {
            let r = regions.with(|r| r.get(i).cloned())?;
            let s = layout.with(|l| l.shapes.get(i).cloned())?;
            let colour = colour(r.colour.unwrap_or(0));
            Some((r.name, measure_tip.get_value(), r.value, colour, s.centre))
        } else {
            let p = points.with(|p| p.get(i - n).cloned())?;
            let at = layout.with(|l| l.points.get(i - n).copied().flatten())?;
            Some((p.name, "Value".to_string(), p.value, colour(p.group), at))
        }
    };
    let text =
        move |i: usize| mark(i).map(|(name, _, v, _, _)| format!("{name}: {}", value_text(v, fmt))).unwrap_or_default();
    let aria = nav::hinted(&label, nav::HINT_REGIONS);
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let (n, _) = counts();
        let (name, _, value, _, _) = mark(i)?;
        let (series, index) = if i < n { (0, i) } else { (1, i - n) };
        Some(EventDatum { series, index, label: Some(name), value, node: None })
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let measure_head = measure;

    view! {
        <figure data-scope="geo-map" data-part="root" data-active=move || active.get().map(|_| "")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=move || layout.with(|l| format!("0 0 {} {:.2}", l.width, l.height))
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
                        let (n, m) = counts();
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), n + m) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        regions
                            .get()
                            .into_iter()
                            .zip(l.shapes)
                            .enumerate()
                            .map(|(i, (r, s))| {
                                view! {
                                    <path
                                        data-part="region"
                                        data-empty=(s.t.is_none() && r.colour.is_none()).then_some("")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        d=s.path
                                        style=geo::region_style(&r, s.t, i)
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                    ></path>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let (n, _) = counts();
                        let l = layout.get();
                        points
                            .get()
                            .into_iter()
                            .zip(l.points)
                            .enumerate()
                            .filter_map(|(k, (p, at))| at.map(|at| (k, p, at)))
                            .map(|(k, p, (x, y))| {
                                let i = n + k;
                                view! {
                                    <circle
                                        data-part="point"
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        cx=format!("{x:.2}")
                                        cy=format!("{y:.2}")
                                        r="5"
                                        style=format!("fill: {}; --i: {};", colour(p.group), k.min(40))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                    ></circle>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let (name, row, v, colour, (x, y)) = mark(active.get()?)?;
                    let (left, top) = layout.with(|l| (x / l.width * 100.0, y / l.height * 100.0));
                    // The tooltip opens below the mark near the top edge (top < 25%), above it elsewhere.
                    let tip = Tip {
                        x: left / 100.0,
                        y: if top < 25.0 { 1.0 } else { 0.0 },
                        title: name,
                        rows: vec![TipRow::new(None, row, value_text(v, fmt))],
                    };
                    let style = format!("left: {left:.2}%; top: {top:.2}%;");
                    Some(view! { <ChartTip tip swatch=colour style /> })
                }}
            </div>
            <div data-part="legend" aria-hidden="true">
                <output data-part="readout">{move || active.get().map_or_else(|| "Hover or focus a region".to_string(), text)}</output>
                <Show when=move || regions.with(|r| r.iter().any(|r| r.colour.is_none() && r.value.is_finite()))>
                    <div data-part="scale">
                        <span>{move || fmt(layout.with(|l| l.range.0))}</span>
                        <span data-part="ramp"></span>
                        <span>{move || fmt(layout.with(|l| l.range.1))}</span>
                    </div>
                </Show>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(text).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr><th scope="col">"Name"</th><th scope="col">"Kind"</th><th scope="col">{measure_head}</th></tr>
                </thead>
                <tbody>
                    {move || {
                        regions
                            .get()
                            .into_iter()
                            .map(|r| (r.name, "Region", r.value))
                            .chain(points.get().into_iter().map(|p| (p.name, "Point", p.value)))
                            .map(|(name, kind, v)| view! { <tr><th scope="row">{name}</th><td>{kind}</td><td>{value_text(v, fmt)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
