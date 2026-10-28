//! `<ComposedChart>`: bars, lines, areas and points over shared categories and one value
//! axis, with a crosshair tooltip that reads every layer. Geometry comes from
//! `hydroxy_charts::composed`.

use hydroxy_charts::composed::LayerKind;
use hydroxy_charts::nav::{self, HINT_READ_VALUES_LR, hinted};
use hydroxy_charts::{annotate, bar, composed, format_value, line};
use leptos::prelude::*;

use crate::line_chart::{annotation_views, plot_fractions};
use crate::plot_kit::{colour, pct};
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::composed::Layer;
pub use hydroxy_charts::line::Curve;

/// Layers of bars, lines, areas and points over one set of categories. Hover, touch or
/// focus the plot and use Left/Right to read every layer at one category.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn ComposedChart(
    /// The layers, drawn in order (later ones on top) in categorical colours.
    /// `Layer::bar`, `Layer::line`, `Layer::area` or `Layer::points`.
    #[prop(into)]
    layers: Signal<Vec<Layer>>,
    /// One label per category, used on the axis, tooltip and table.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows, such as "Orders against target". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 240)]
    height: u16,
    /// How line and area layers join their points.
    #[prop(optional)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` values are category indices.
    /// The value axis grows to fit them.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        let extent = annotations.with(|a| annotate::y_extent(a));
        layers.with(|l| composed::layout(l, labels.with(Vec::len), curve, 5, extent))
    });
    let placed = Memo::new(move |_| {
        let (n, y) = (labels.with(Vec::len), layout.with(|l| l.scale));
        annotations.with(|a| annotate::place(a, |i| bar::category_centre(i, n), |v| y.map(v)))
    });
    let active = RwSignal::new(None::<usize>);
    let value_at = move |l: &Layer, i: usize| l.value(i).map_or_else(|| "–".to_string(), fmt);
    let label_at = move |i: usize| labels.with(|l| l.get(i).cloned().unwrap_or_default());
    let move_to = move |ev: leptos::ev::PointerEvent| {
        let Some((fx, _)) = plot_fractions(&ev) else { return };
        let i = layout.with(|l| line::nearest_index(&l.centres, fx));
        if active.get_untracked() != i {
            active.set(i);
        }
    };
    let announcement = move || {
        active.get().map_or_else(String::new, |i| {
            let parts: Vec<String> =
                layers.with(|ls| ls.iter().map(|l| format!("{} {}", l.name, value_at(l, i))).collect());
            format!("{}: {}", label_at(i), parts.join(", "))
        })
    };

    view! {
        <figure data-scope="composed-chart" data-part="root">
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .gridlines
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div data-part="gridline" data-baseline=(g.value == 0.0).then_some("") style=format!("bottom: {};", pct(g.fraction))>
                                        <span data-part="tick">{fmt(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div data-part="annotations" aria-hidden="true">{move || annotation_views(placed.get())}</div>
                <div data-part="bars" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .layers
                            .into_iter()
                            .enumerate()
                            .flat_map(|(li, g)| g.bars.into_iter().enumerate().filter_map(move |(i, b)| b.map(|b| (li, i, b))))
                            .map(|(li, i, b)| {
                                view! {
                                    <div
                                        data-part="bar"
                                        data-index=li
                                        data-negative=b.negative.then_some("")
                                        style=format!(
                                            "left: {}; width: {}; bottom: {}; height: {}; background: {}; --i: {i};",
                                            pct(b.left),
                                            pct(b.width),
                                            pct(b.bottom),
                                            pct(b.height),
                                            colour(li),
                                        )
                                    ></div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                    {move || {
                        let kinds: Vec<LayerKind> = layers.with(|l| l.iter().map(|l| l.kind).collect());
                        layout
                            .get()
                            .layers
                            .into_iter()
                            .enumerate()
                            .filter(|(i, _)| matches!(kinds.get(*i), Some(LayerKind::Line | LayerKind::Area)))
                            .map(|(i, g)| {
                                let c = colour(i);
                                view! {
                                    <g data-part="series" data-index=i style=format!("--i: {i};")>
                                        {(!g.area.is_empty()).then(|| view! { <path data-part="area" d=g.area fill=c.clone()></path> })}
                                        <path data-part="line" d=g.line stroke=c pathLength="1"></path>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                <div data-part="points" aria-hidden="true">
                    {move || {
                        let kinds: Vec<LayerKind> = layers.with(|l| l.iter().map(|l| l.kind).collect());
                        let l = layout.get();
                        l.layers
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| kinds.get(*i) == Some(&LayerKind::Points))
                            .flat_map(|(li, g)| {
                                g.points.iter().zip(&l.centres).enumerate().filter_map(move |(i, (p, &x))| p.map(|f| (li, i, x, f)))
                            })
                            .map(|(li, i, x, f)| {
                                view! {
                                    <span data-part="point" style=format!("left: {}; bottom: {}; background: {}; --i: {i};", pct(x), pct(f), colour(li))></span>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div
                    data-part="overlay"
                    tabindex="0"
                    role="group"
                    aria-label=hinted(&label, HINT_READ_VALUES_LR)
                    on:pointermove=move_to
                    on:pointerdown=move_to
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), labels.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let i = active.get()?;
                        let l = layout.get();
                        let x = *l.centres.get(i)?;
                        let dots = l
                            .layers
                            .iter()
                            .enumerate()
                            .filter_map(|(li, g)| {
                                g.points.get(i).copied().flatten().map(|f| {
                                    view! { <span data-part="dot" style=format!("left: {}; bottom: {}; background: {};", pct(x), pct(f), colour(li))></span> }
                                })
                            })
                            .collect_view();
                        let rows = layers
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(li, layer)| {
                                view! {
                                    <li data-part="tooltip-row">
                                        <span data-part="swatch" data-kind=layer.kind.attr() style=format!("background: {};", colour(li))></span>
                                        <span data-part="tooltip-name">{layer.name.clone()}</span>
                                        <span data-part="tooltip-value">{value_at(&layer, i)}</span>
                                    </li>
                                }
                            })
                            .collect_view();
                        Some(
                            view! {
                                <div data-part="crosshair" style=format!("left: {};", pct(x))></div>
                                {dots}
                                <div data-part="tooltip" data-side={if x > 0.5 { "left" } else { "right" }} style=format!("left: {};", pct(x)) aria-hidden="true">
                                    <div data-part="tooltip-label">{label_at(i)}</div>
                                    <ul>{rows}</ul>
                                </div>
                            },
                        )
                    }}
                </div>
            </div>
            <div data-part="x-labels" aria-hidden="true">
                {move || {
                    let (l, centres) = (labels.get(), layout.with(|l| l.centres.clone()));
                    line::label_indices(l.len(), 8)
                        .into_iter()
                        .map(|i| view! { <span data-part="x-label" style=format!("left: {};", pct(centres[i]))>{l[i].clone()}</span> })
                        .collect_view()
                }}
            </div>
            <Show when=move || layers.with(|l| l.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        layers
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, l)| {
                                view! {
                                    <li data-part="legend-item" data-index=i>
                                        <span data-part="swatch" data-kind=l.kind.attr() style=format!("background: {};", colour(i))></span>
                                        {l.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{announcement}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <td></td>
                        {move || layers.get().into_iter().map(|l| view! { <th scope="col">{l.name}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let ls = layers.get();
                        labels
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, name)| {
                                let cells = ls.iter().map(|l| view! { <td>{value_at(l, i)}</td> }).collect_view();
                                view! { <tr><th scope="row">{name}</th>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
