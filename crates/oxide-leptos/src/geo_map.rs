//! `<GeoMap>`: a choropleth over caller-supplied polygons. Geometry comes from
//! `oxide_charts::geo`; no map data is bundled. Regions take hover and focus.

use leptos::prelude::*;
use oxide_charts::{format_value, geo};

pub use oxide_charts::geo::{Projection, Region};

#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
/// Regions shaded on a sequential scale by value. Hover or focus a region to read it.
#[component]
pub fn GeoMap(
    /// The regions, as rings of `(longitude, latitude)` degrees.
    #[prop(into)]
    regions: Signal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Name of the measure, such as "Customers per 1k people". Heads the table column.
    #[prop(into)]
    measure: String,
    /// Equirectangular (default) or Mercator.
    #[prop(optional)]
    projection: Projection,
    /// Formats values on the legend, readout and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| regions.with(|r| geo::layout(r, projection)));
    let active = RwSignal::new(None::<usize>);
    let text = move |i: usize| regions.with(|r| r.get(i).map_or_else(String::new, |r| format!("{}: {}", r.name, if r.value.is_finite() { fmt(r.value) } else { "no data".to_string() })));
    let aria = label.clone();
    let measure_head = measure.clone();
    view! {
        <figure data-scope="geo-map" data-part="root">
            {move || {
                let l = layout.get();
                let paths = l
                    .shapes
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        view! {
                            <path
                                data-part="region"
                                data-empty=s.t.is_none().then_some("")
                                data-active=move || (active.get() == Some(i)).then_some("")
                                d=s.path.clone()
                                style=s.t.map(|t| format!("--t: {t:.3};"))
                                tabindex="0"
                                role="img"
                                aria-label=text(i)
                                on:pointerenter=move |_| active.set(Some(i))
                                on:pointerleave=move |_| active.set(None)
                                on:focus=move |_| active.set(Some(i))
                                on:blur=move |_| active.set(None)
                            ></path>
                        }
                    })
                    .collect_view();
                view! {
                    <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) role="group" aria-label=aria.clone()>
                        {paths}
                    </svg>
                }
            }}
            <div data-part="legend" aria-hidden="true">
                <output data-part="readout">{move || active.get().map_or_else(|| "Hover or focus a region".to_string(), text)}</output>
                <div data-part="scale">
                    <span>{move || fmt(layout.with(|l| l.range.0))}</span>
                    <span data-part="ramp"></span>
                    <span>{move || fmt(layout.with(|l| l.range.1))}</span>
                </div>
            </div>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr><th scope="col">"Region"</th><th scope="col">{measure_head}</th></tr>
                </thead>
                <tbody>
                    {move || {
                        regions
                            .get()
                            .into_iter()
                            .map(|r| view! { <tr><th scope="row">{r.name}</th><td>{if r.value.is_finite() { fmt(r.value) } else { "–".to_string() }}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
