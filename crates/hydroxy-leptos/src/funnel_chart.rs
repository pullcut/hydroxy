//! `<FunnelChart>`: stages that narrow to the next, with the conversion between each
//! pair. Geometry comes from `hydroxy_charts::funnel`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, funnel, nav};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour, pct};

pub use hydroxy_charts::{Datum, Orientation};

fn percent(f: f64) -> String {
    format!("{}%", format_value(f * 100.0))
}

/// A funnel, top to bottom or left to right. Hover, touch or focus a stage to read it;
/// focus the chart and use the arrow keys to step through stages, Home and End to
/// jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn FunnelChart(
    /// The stages, in order.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Signup funnel". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Which way the funnel runs. Default top to bottom.
    #[prop(optional)]
    orientation: Orientation,
    /// Plot height in px.
    #[prop(default = 280)]
    height: u16,
    /// Formats values for the labels, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a stage, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a stage.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let stages =
        Memo::new(move |_| data.with(|d| funnel::layout(&d.iter().map(|x| x.value).collect::<Vec<_>>(), 0.03)));
    let active = RwSignal::new(None::<usize>);
    let datum = Signal::derive(move || {
        let i = active.get()?;
        data.with(|d| d.get(i).map(|d| EventDatum::category(i, d.label.clone(), d.value)))
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let len = move || data.with(Vec::len);
    let arrow = if orientation == Orientation::Vertical { "\u{2193}" } else { "\u{2192}" };
    let describe = move |i: usize| {
        let d = data.with(|d| d.get(i).cloned())?;
        let s = stages.with(|s| s.get(i).copied())?;
        let mut text = format!("{}: {}", d.label, fmt(d.value));
        if let Some(c) = s.conversion {
            text = format!("{text}, {} of the previous stage", percent(c));
        }
        Some(text)
    };
    view! {
        <figure data-scope="funnel-chart" data-part="root" data-orientation=orientation.attr() style=move || format!("--ox-segments: {};", len())>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to step through stages.")
                style=format!("height: {height}px;")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                on:keydown=move |ev| {
                    let key = ev.key();
                    if key == "Enter" {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active.get_untracked(), len()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <div data-part="labels" aria-hidden="true">
                    {move || {
                        let s = stages.get();
                        data.get()
                            .into_iter()
                            .zip(s)
                            .enumerate()
                            .map(|(i, (d, s))| {
                                view! {
                                    <div
                                        data-part="stage-label"
                                        data-index=i
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--at: {}; --i: {i};", pct(f64::midpoint(s.start, s.end)))
                                        on:pointerenter=move |_| active.set(Some(i))
                                    >
                                        <span data-part="stage-name">{d.label}</span>
                                        <span data-part="stage-value">{fmt(d.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div data-part="area">
                    <svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                        {move || {
                            stages
                                .get()
                                .into_iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    view! {
                                        <polygon
                                            data-part="stage"
                                            data-index=i
                                            data-active=move || (active.get() == Some(i)).then_some("")
                                            style=format!("--i: {i};")
                                            points=funnel::polygon(&s, orientation)
                                            on:pointerenter=move |_| active.set(Some(i))
                                            on:pointerdown=move |_| active.set(Some(i))
                                        ></polygon>
                                    }
                                })
                                .collect_view()
                        }}
                    </svg>
                    {move || {
                        stages
                            .get()
                            .into_iter()
                            .enumerate()
                            .filter_map(|(i, s)| s.conversion.map(|c| (i, s, c)))
                            .map(|(i, s, c)| {
                                view! {
                                    <span data-part="conversion" aria-hidden="true" style=format!("--at: {}; --i: {i};", pct(s.start))>
                                        {format!("{arrow} {}", percent(c))}
                                    </span>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let i = active.get()?;
                        let d = data.with(|d| d.get(i).cloned())?;
                        let s = stages.with(|s| s.get(i).copied())?;
                        let mid = pct(f64::midpoint(s.start, s.end));
                        let pos = match orientation {
                            Orientation::Vertical => format!("left: 50%; top: {mid};"),
                            Orientation::Horizontal => format!("left: {mid}; top: 50%;"),
                        };
                        let side = if orientation == Orientation::Horizontal && (s.start + s.end) > 1.0 { "left" } else { "right" };
                        let mut rows = vec![TipRow::new(None, "Value", fmt(d.value))];
                        rows.extend(s.conversion.map(|c| TipRow::new(None, "From previous", percent(c))));
                        if i > 0 {
                            rows.extend(s.of_first.map(|f| TipRow::new(None, "From first", percent(f))));
                        }
                        let tip = Tip { x: 0.0, y: 0.0, title: d.label, rows };
                        Some(view! { <ChartTip tip swatch=colour(0) style=pos side /> })
                    }}
                </div>
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Stage"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"From previous"</th>
                        <th scope="col">"From first"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let s = stages.get();
                        data.get()
                            .into_iter()
                            .zip(s)
                            .map(|(d, s)| {
                                view! {
                                    <tr>
                                        <th scope="row">{d.label}</th>
                                        <td>{fmt(d.value)}</td>
                                        <td>{s.conversion.map(percent).unwrap_or_default()}</td>
                                        <td>{s.of_first.map(percent).unwrap_or_default()}</td>
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
