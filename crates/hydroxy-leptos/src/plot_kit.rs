//! The shared shell for the single-value-axis cartesian charts: gridlines and ticks,
//! annotations, a focusable overlay with a tooltip, category labels, legend, live region
//! and screen-reader table. Each chart supplies its marks and keyboard and pointer handling.

use hydroxy_charts::annotate::{Annotation, Placed};
use hydroxy_charts::event::Datum;
use hydroxy_charts::format_value;
use hydroxy_charts::plot::{self, Key, Tick, Tip};
use leptos::prelude::*;

#[allow(unused_imports, reason = "re-exported for the chart components; not every one uses all three")]
pub(crate) use hydroxy_charts::plot::{colour, colour_or_muted, pct};

/// The tooltip for `tip`, pinned at its anchor in % of the plot (`y` counts from the bottom).
///
/// By default it sets `data-side` and `data-vertical` from the anchor. A chart that places its
/// own tooltip can pass `anchored` (sets `data-anchored`, drops `data-vertical`), its own
/// `style` and `side`, and a `swatch` colour for the title.
#[allow(unreachable_pub, reason = "#[component] makes the generated props struct pub")]
#[component]
pub(crate) fn ChartTip(
    /// Heading and rows.
    tip: Tip,
    /// Swatch colour shown before the title.
    #[prop(optional, into)]
    swatch: Option<String>,
    /// Mark the tooltip as `data-anchored`.
    #[prop(optional)]
    anchored: bool,
    /// Replaces the default `left`/`top` style.
    #[prop(optional, into)]
    style: Option<String>,
    /// Replaces the default `data-side`.
    #[prop(optional)]
    side: Option<&'static str>,
) -> impl IntoView {
    let style = style.unwrap_or_else(|| format!("left: {:.2}%; top: {:.2}%;", tip.x * 100.0, (1.0 - tip.y) * 100.0));
    let side = side.unwrap_or_else(|| tip.side());
    let vertical = (!anchored).then(|| tip.vertical());
    view! {
        <div
            data-part="tooltip"
            data-anchored=anchored.then_some("")
            data-side=side
            data-vertical=vertical
            style=style
            aria-hidden="true"
        >
            <div data-part="tooltip-label">
                {swatch.map(|c| view! { <span data-part="swatch" style=format!("background: {c};")></span> })}
                {tip.title.clone()}
            </div>
            <ul>
                {tip
                    .rows
                    .into_iter()
                    .map(|r| {
                        view! {
                            <li data-part="tooltip-row" data-active=r.active.then_some("")>
                                <span data-part="tooltip-name">
                                    {r.colour.map(|c| view! { <span data-part="swatch" style=format!("background: {c};")></span> })}
                                    {r.name}
                                </span>
                                <span data-part="tooltip-value">{r.value}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </div>
    }
}

/// The chart's formatter, or the plain default.
pub(crate) fn formatter(
    format: Option<Callback<f64, String>>,
) -> impl Fn(f64) -> String + Copy + Send + Sync + 'static {
    move |v| format.map_or_else(|| format_value(v), |f| f.run(v))
}

/// Annotations placed for the plot, as a signal.
pub(crate) fn placed(
    f: impl Fn(&[Annotation]) -> Vec<Placed> + Send + Sync + 'static,
    annotations: Signal<Vec<Annotation>>,
) -> Signal<Vec<Placed>> {
    Signal::derive(move || annotations.with(|a| f(a)))
}

#[allow(unreachable_pub, reason = "#[component] makes the generated props struct pub")]
#[component]
pub(crate) fn Plot(
    scope: &'static str,
    label: String,
    /// How to use the keyboard, appended to the accessible name.
    hint: &'static str,
    height: u16,
    #[prop(optional)] horizontal: bool,
    #[prop(into)] ticks: Signal<Vec<Tick>>,
    #[prop(optional, into)] right_ticks: Signal<Vec<Tick>>,
    #[prop(into)] labels: Signal<Vec<Tick>>,
    #[prop(into)] annotations: Signal<Vec<Placed>>,
    #[prop(optional, into)] legend: Signal<Vec<Key>>,
    #[prop(into)] tip: Signal<Option<Tip>>,
    #[prop(into)] head: Signal<Vec<String>>,
    #[prop(into)] rows: Signal<Vec<Vec<String>>>,
    on_key: Callback<String, bool>,
    on_point: Callback<Option<(f64, f64)>>,
    #[prop(into)] marks: ViewFn,
    #[prop(optional, into)] overlay: ViewFn,
    /// A chart-specific mode, as `data-variant`.
    #[prop(optional)]
    variant: &'static str,
    /// The chart's active mark as an event [`Datum`], for `on_hover` and `on_select`.
    #[prop(optional_no_strip)]
    datum: Option<Signal<Option<Datum>>>,
    /// Called when the active mark changes (pointer or keyboard); `None` when it clears.
    #[prop(optional_no_strip)]
    on_hover: Option<Callback<Option<Datum>>>,
    /// Called on click, tap or Enter on the active mark.
    #[prop(optional_no_strip)]
    on_select: Option<Callback<Datum>>,
) -> impl IntoView {
    crate::callbacks::watch_hover(datum, on_hover);
    let point = move |ev: leptos::ev::PointerEvent| {
        let Some(el) =
            ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok())
        else {
            return;
        };
        let (w, h) = (f64::from(el.offset_width().max(1)), f64::from(el.offset_height().max(1)));
        on_point.run(Some((f64::from(ev.offset_x()) / w, 1.0 - f64::from(ev.offset_y()) / h)));
    };
    let tick = |t: Tick, axis: &'static str| {
        view! {
            <div data-part="gridline" data-axis=axis data-baseline=t.baseline.then_some("") style=format!("--p: {};", pct(t.at))>
                <span data-part="tick">{t.text}</span>
            </div>
        }
    };
    let caption = label.clone();

    view! {
        <figure
            data-scope=scope
            data-part="root"
            data-plot=""
            data-orientation=if horizontal { "horizontal" } else { "vertical" }
            data-right-axis=move || right_ticks.with(|t| !t.is_empty()).then_some("")
            data-variant=(!variant.is_empty()).then_some(variant)
            style=format!("--ox-height: {height}px;")
        >
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || ticks.get().into_iter().map(|t| tick(t, "value")).collect_view()}
                    {move || right_ticks.get().into_iter().map(|t| tick(t, "right")).collect_view()}
                </div>
                {move || marks.run()}
                {move || {
                    annotations
                        .get()
                        .into_iter()
                        .map(|a| {
                            let style = a.style();
                            view! {
                                <div data-part="annotation" data-shape=a.shape.attr() style=style aria-hidden="true">
                                    {a.label.map(|l| view! { <span data-part="annotation-label">{l}</span> })}
                                </div>
                            }
                        })
                        .collect_view()
                }}
                <div
                    data-part="overlay"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. {hint}")
                    on:pointermove=point
                    on:pointerdown=point
                    on:pointerleave=move |_| on_point.run(None)
                    on:blur=move |_| on_point.run(None)
                    on:click=move |_| crate::callbacks::select_active(datum, on_select)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            crate::callbacks::select_active(datum, on_select);
                        }
                        if on_key.run(ev.key()) {
                            ev.prevent_default();
                        }
                    }
                >
                    {move || overlay.run()}
                    {move || {
                        tip.get()
                            .map(|t| {
                                let rows = t
                                    .rows
                                    .iter()
                                    .cloned()
                                    .map(|r| {
                                        view! {
                                            <li data-part="tooltip-row" data-active=r.active.then_some("")>
                                                {r.colour.map(|c| view! { <span data-part="swatch" style=format!("background: {c};")></span> })}
                                                <span data-part="tooltip-name">{r.name}</span>
                                                <span data-part="tooltip-value">{r.value}</span>
                                            </li>
                                        }
                                    })
                                    .collect_view();
                                view! {
                                    <div
                                        data-part="tooltip"
                                        data-side=t.side()
                                        data-vertical=t.vertical()
                                        style=format!("left: {}; bottom: {};", pct(t.x), pct(t.y))
                                        aria-hidden="true"
                                    >
                                        <div data-part="tooltip-label">{t.title.clone()}</div>
                                        <ul>{rows}</ul>
                                    </div>
                                }
                            })
                    }}
                </div>
            </div>
            <div data-part="x-labels" aria-hidden="true" style=move || labels.with(|l| format!("--room: {};", pct(plot::tick_room(l))))>
                {move || {
                    labels
                        .get()
                        .into_iter()
                        .map(|t| {
                            let title = t.text.clone();
                            view! { <span data-part="x-label" title=title style=format!("--p: {};", pct(t.at))>{t.text}</span> }
                        })
                        .collect_view()
                }}
            </div>
            <Show when=move || legend.with(|l| !l.is_empty())>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        legend
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, k)| {
                                view! {
                                    <li data-part="legend-item" data-index=i>
                                        <span data-part="swatch" data-dashed=k.dashed.then_some("") style=format!("--swatch: {};", k.colour)></span>
                                        {k.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || tip.with(|t| t.as_ref().map(Tip::announce).unwrap_or_default())}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>{move || head.get().into_iter().map(|h| view! { <th scope="col">{h}</th> }).collect_view()}</tr>
                </thead>
                <tbody>
                    {move || {
                        rows.get()
                            .into_iter()
                            .map(|r| {
                                let mut cells = r.into_iter();
                                let first = cells.next().unwrap_or_default();
                                view! {
                                    <tr>
                                        <th scope="row">{first}</th>
                                        {cells.map(|c| view! { <td>{c}</td> }).collect_view()}
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
