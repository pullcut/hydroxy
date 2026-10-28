//! `<HexbinMapChart>`: points binned into hexagons over projected map shapes, each
//! hexagon shaded by how many points it holds. Geometry comes from `hydroxy_charts::geo`
//! and `hydroxy_charts::hexbin`; no map data is bundled.

use hydroxy_charts::{format_value, geo, hexbin, nav};
use leptos::prelude::*;

pub use hydroxy_charts::geo::{GeoPoint, Projection, Region};

/// Hexagons over a map, darker where more points fall. Hover, touch or focus the map and
/// use the arrow keys to step through the hexagons from the top left, Home and End to
/// jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn HexbinMapChart(
    /// The points to bin. Their values add up into each hexagon's total.
    #[prop(into)]
    points: Signal<Vec<GeoPoint>>,
    /// Base shapes drawn under the hexagons, as rings of `(longitude, latitude)`. Their values are ignored.
    #[prop(optional, into)]
    regions: Signal<Vec<Region>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Hexagon radius, centre to corner, in viewBox units (the map is 640 wide).
    #[prop(default = 16.0)]
    radius: f64,
    /// Equirectangular (default), Mercator, or `Projection::Custom(fn)`.
    #[prop(optional)]
    projection: Projection,
    /// Formats each hexagon's total value. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| regions.with(|r| points.with(|p| geo::layout(r, p, projection))));
    let bins = Memo::new(move |_| points.with(|p| layout.with(|l| hexbin::cells(p, &l.points, radius))));
    let active = RwSignal::new(None::<usize>);
    let text = move |i: usize| {
        bins.with(|b| {
            b.get(i).map(|c| {
                format!(
                    "{}: {} {}, total {}",
                    c.title,
                    c.count,
                    if c.count == 1 { "point" } else { "points" },
                    fmt(c.total)
                )
            })
        })
        .unwrap_or_default()
    };
    let aria = format!("{label}. Use the arrow keys to step through hexagons.");

    view! {
        <figure data-scope="hexbin-map-chart" data-part="root" data-active=move || active.get().map(|_| "")>
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
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), bins.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    <g data-part="land" aria-hidden="true">
                        {move || layout.get().shapes.into_iter().map(|s| view! { <path d=s.path></path> }).collect_view()}
                    </g>
                    {move || {
                        bins.get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, c)| {
                                view! {
                                    <path
                                        data-part="hex"
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        d=c.path
                                        style=format!("--t: {:.3}; --i: {};", c.t, i.min(40))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                    ></path>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let c = bins.with(|b| b.get(active.get()?).cloned())?;
                    let (left, top) = layout.with(|l| (c.x / l.width * 100.0, c.y / l.height * 100.0));
                    Some(view! {
                        <div
                            data-part="tooltip"
                            data-side={if left > 50.0 { "left" } else { "right" }}
                            data-vertical={if top < 25.0 { "below" } else { "above" }}
                            style=format!("left: {left:.2}%; top: {top:.2}%;")
                            aria-hidden="true"
                        >
                            <div data-part="tooltip-label">
                                <span data-part="swatch" style=format!("background: color-mix(in oklab, var(--chart-1) {:.0}%, var(--surface-sunken));", 15.0 + c.t * 85.0)></span>
                                {c.title}
                            </div>
                            <ul>
                                <li data-part="tooltip-row">
                                    <span data-part="tooltip-name">"Points"</span>
                                    <span data-part="tooltip-value">{c.count}</span>
                                </li>
                                <li data-part="tooltip-row">
                                    <span data-part="tooltip-name">"Total"</span>
                                    <span data-part="tooltip-value">{fmt(c.total)}</span>
                                </li>
                            </ul>
                        </div>
                    })
                }}
            </div>
            <div data-part="scale" aria-hidden="true">
                <span>"1"</span>
                <span data-part="ramp"></span>
                <span>{move || bins.with(|b| b.iter().map(|c| c.count).max().unwrap_or(1))}</span>
                <span data-part="scale-label">"points per hexagon"</span>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().map(text).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Hexagon"</th>
                        <th scope="col">"Points"</th>
                        <th scope="col">"Total"</th>
                        <th scope="col">"Includes"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        bins.get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, c)| view! {
                                <tr>
                                    <th scope="row">{format!("Hexagon {}", i + 1)}</th>
                                    <td>{c.count}</td>
                                    <td>{fmt(c.total)}</td>
                                    <td>{c.names}</td>
                                </tr>
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
