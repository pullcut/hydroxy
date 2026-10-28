//! The frame shared by the distribution and financial charts: gridlines and ticks,
//! annotations, the focusable overlay with its marker and tooltip, legend, live region
//! and screen-reader table. Each chart passes its marks as children.

use hydroxy_charts::event::Datum;
use hydroxy_charts::probe::{self, Pick, Readout};
use hydroxy_charts::{format_value, nav};
use leptos::prelude::*;

pub(crate) use crate::plot_kit::{colour, pct};

/// The shared frame. `active` is the focused stop, so marks can highlight it too.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub(crate) fn PlotShell(
    /// `data-scope` of the root.
    scope: &'static str,
    /// Accessible name and table caption.
    label: String,
    /// How to use the keyboard, appended to the overlay's name.
    hint: &'static str,
    /// Plot height in px.
    height: u16,
    /// Ticks, stops, legend, notes and table.
    #[prop(into)]
    readout: Signal<Readout>,
    /// Formats values.
    format: Option<Callback<f64, String>>,
    /// The focused stop.
    active: RwSignal<Option<usize>>,
    /// The marks.
    children: Children,
    /// The focused stop as an event [`Datum`], for `on_hover` and `on_select`.
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
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let current = move || active.get().and_then(|i| readout.with(|r| r.stops.get(i).cloned()));
    // Browsers fire a pointermove when content changes under a still cursor; ignore it
    // so it can't clear a keyboard selection.
    let last = StoredValue::new((i32::MIN, i32::MIN));
    let move_to = move |ev: leptos::ev::PointerEvent| {
        let at = (ev.client_x(), ev.client_y());
        if last.get_value() == at {
            return;
        }
        last.set_value(at);
        let Some(el) =
            ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok())
        else {
            return;
        };
        let (w, h) = (f64::from(el.offset_width().max(1)), f64::from(el.offset_height().max(1)));
        let (x, y) = (f64::from(ev.offset_x()) / w, 1.0 - f64::from(ev.offset_y()) / h);
        let near = readout.with(|r| probe::nearest(&r.stops, x, y, r.pick));
        if active.get_untracked() != near {
            active.set(near);
        }
    };
    let caption = label.clone();

    view! {
        <figure data-scope=scope data-part="root">
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        readout
                            .with(|r| r.y_ticks.clone())
                            .into_iter()
                            .map(|t| {
                                view! {
                                    <div data-part="gridline" data-axis="y" style=format!("bottom: {};", pct(t.fraction))>
                                        <span data-part="tick">{t.label.show(fmt)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let bands = readout.with(|r| r.x_bands).then_some("");
                        readout
                            .with(|r| r.x_ticks.clone())
                            .into_iter()
                            .map(|t| {
                                view! {
                                    <div data-part="gridline" data-axis="x" data-band=bands style=format!("left: {};", pct(t.fraction))>
                                        <span data-part="tick">{t.label.show(fmt)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                {children()}
                {move || {
                    readout
                        .with(|r| r.notes.clone())
                        .into_iter()
                        .map(|n| {
                            let style = n.style();
                            view! {
                                <div data-part="annotation" data-shape=n.shape.attr() style=style aria-hidden="true">
                                    {n.label.map(|l| view! { <span data-part="annotation-label">{l}</span> })}
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
                    on:pointermove=move_to
                    on:pointerdown=move_to
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:click=move |_| crate::callbacks::select_active(datum, on_select)
                    on:keydown=move |ev| {
                        let key = ev.key();
                        if key == "Enter" {
                            crate::callbacks::select_active(datum, on_select);
                        }
                        let key = if readout.with(|r| r.pick) == Pick::Row { nav::linear_key(&key) } else { key.as_str() };
                        if let Some(next) = nav::list_key(key, active.get_untracked(), readout.with(|r| r.stops.len())) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        current()
                            .map(|s| {
                                let swatch = s.colour.map(|c| view! { <span data-part="swatch" style=format!("background: {};", colour(c))></span> });
                                let rows = s
                                    .rows
                                    .iter()
                                    .map(|r| {
                                        let swatch = r.colour.map(|c| view! { <span data-part="swatch" style=format!("background: {};", colour(c))></span> });
                                        view! {
                                            <li data-part="tooltip-row">
                                                <span data-part="tooltip-name">{swatch}{r.name.clone()}</span>
                                                <span data-part="tooltip-value">{r.value.show(fmt)}</span>
                                            </li>
                                        }
                                    })
                                    .collect_view();
                                let at = format!("left: {}; bottom: {};", pct(s.x), pct(s.y));
                                view! {
                                    <span data-part="marker" style=format!("{at} background: {};", colour(s.colour.unwrap_or(0)))></span>
                                    <div
                                        data-part="tooltip"
                                        data-side={if s.x > 0.5 { "left" } else { "right" }}
                                        data-vertical={if s.y > 0.5 { "below" } else { "above" }}
                                        style=at
                                        aria-hidden="true"
                                    >
                                        <div data-part="tooltip-label">{swatch}{s.title.show(fmt)}</div>
                                        <ul>{rows}</ul>
                                    </div>
                                }
                            })
                    }}
                </div>
            </div>
            <Show when=move || readout.with(|r| !r.legend.is_empty())>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        readout
                            .with(|r| r.legend.clone())
                            .into_iter()
                            .map(|(name, c)| {
                                view! {
                                    <li data-part="legend-item">
                                        <span data-part="swatch" style=format!("background: {};", colour(c))></span>
                                        {name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || current().map(|s| probe::describe(&s, fmt)).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>{move || readout.with(|r| r.head.clone()).into_iter().map(|h| view! { <th scope="col">{h}</th> }).collect_view()}</tr>
                </thead>
                <tbody>
                    {move || {
                        readout
                            .with(|r| r.rows.clone())
                            .into_iter()
                            .map(|row| {
                                let mut cells = row.into_iter();
                                let head = cells.next().map(|c| c.show(fmt)).unwrap_or_default();
                                view! {
                                    <tr>
                                        <th scope="row">{head}</th>
                                        {cells.map(|c| view! { <td>{c.show(fmt)}</td> }).collect_view()}
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
