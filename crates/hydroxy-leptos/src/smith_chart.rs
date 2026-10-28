//! `<SmithChart>`: impedance samples on the Smith chart, for RF work. Geometry comes
//! from `hydroxy_charts::smith`. Hover a sample or a legend row to pick out its trace.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{nav, smith};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::smith::Sample;

/// Pointer reach for picking a sample, in drawing units.
const REACH: f64 = 24.0;

fn impedance(s: &Sample, fmt: impl Fn(f64) -> String) -> String {
    format!("{} {} j{}", fmt(s.real), if s.imaginary < 0.0 { "−" } else { "+" }, fmt(s.imaginary.abs()))
}

/// Impedance sweeps inside the unit disc, with the resistance and reactance grid.
/// Samples that share a `trace` name form one swept trace; their first and last labels name its ends.
/// Hover or touch to read the nearest sample; focus the chart and use Left and Right to
/// step along a trace, Up and Down to switch trace, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn SmithChart(
    /// The samples, in sweep order within each trace.
    #[prop(into)]
    samples: Signal<Vec<Sample>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Reference impedance the samples are measured against. 1 means already normalised;
    /// pass 50 when the samples are raw ohms.
    #[prop(default = 1.0)]
    z0: f64,
    /// Formats resistance and reactance in the tooltip and table. Default: two decimal places.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called with the sample under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the sample when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format!("{v:.2}"), |f| f.run(v));
    let layout = Memo::new(move |_| samples.with(|s| smith::layout(s, z0)));
    let active = RwSignal::new(None::<(usize, usize)>);
    let hovered_trace = RwSignal::new(None::<usize>);
    let highlight = move || hovered_trace.get().or_else(|| active.get().map(|(t, _)| t));
    let sample_at = move |(t, k): (usize, usize)| {
        layout.with(|l| {
            l.traces.get(t).and_then(|tr| Some((tr.name.clone(), tr.samples.get(k)?.clone(), *tr.points.get(k)?)))
        })
    };
    // `series` is the trace, `index` the sample within it and `value` its resistance.
    let datum = Signal::derive(move || {
        let (t, k) = active.get()?;
        let (name, s, _) = sample_at((t, k))?;
        Some(EventDatum { series: t, index: k, label: Some(s.label.unwrap_or(name)), value: s.real, node: None })
    });
    watch_hover(Some(datum), on_hover);
    let describe = move |at: (usize, usize)| {
        sample_at(at).map(|(name, s, _)| match &s.label {
            Some(l) => format!("{name}, {l}: {}", impedance(&s, fmt)),
            None => format!("{name}: {}", impedance(&s, fmt)),
        })
    };
    let pick = move |ev: leptos::ev::PointerEvent| {
        let Some(el) =
            ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok())
        else {
            return;
        };
        let near = layout.with(|l| {
            let scale = l.width / f64::from(el.offset_width().max(1));
            smith::nearest(l, f64::from(ev.offset_x()) * scale, f64::from(ev.offset_y()) * scale, REACH)
        });
        if active.get_untracked() != near {
            active.set(near);
        }
    };
    let on_key = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            select_active(Some(datum), on_select);
        }
        let next = layout.with(|l| {
            let cols = l.traces.iter().map(|t| t.points.len()).max().unwrap_or(0);
            nav::grid_key(&ev.key(), active.get_untracked(), l.traces.len(), cols)
                .map(|n| n.map(|(t, k)| (t, k.min(l.traces[t].points.len().saturating_sub(1)))))
        });
        if let Some(next) = next {
            ev.prevent_default();
            active.set(next);
        }
    };
    let group_label = format!("{label}. Use the arrow keys to step through samples; Up and Down switch trace.");
    let caption = label.clone();

    view! {
        <figure data-scope="smith-chart" data-part="root" data-highlight=move || highlight().map(|_| "")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointermove=pick
                on:pointerdown=pick
                on:click=move |_| select_active(Some(datum), on_select)
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=on_key
            >
                {move || {
                    let l = layout.get();
                    let grid = l
                        .resistance
                        .iter()
                        .map(|&(x, y, r)| view! { <circle data-part="grid" cx=x cy=y r=r></circle> })
                        .collect_view();
                    let arcs = l.reactance.iter().map(|d| view! { <path data-part="grid" d=d.clone()></path> }).collect_view();
                    let traces = l
                        .traces
                        .iter()
                        .enumerate()
                        .map(|(i, t)| {
                            let dots = t
                                .points
                                .iter()
                                .enumerate()
                                .map(|(k, &(x, y))| {
                                    let end = k == 0 || k + 1 == t.points.len();
                                    view! {
                                        <circle
                                            data-part="point"
                                            data-end=end.then_some("")
                                            data-active=move || (active.get() == Some((i, k))).then_some("")
                                            cx=x
                                            cy=y
                                            r=if end { "4.5" } else { "3" }
                                        ></circle>
                                    }
                                })
                                .collect_view();
                            let ends = [(0, t.samples.first()), (t.points.len().saturating_sub(1), t.samples.last())]
                                .into_iter()
                                .filter(|_| t.points.len() > 1)
                                .filter_map(|(k, s)| {
                                    let text = s?.label.clone()?;
                                    let (x, y) = t.points[k];
                                    let right = x >= l.centre.0;
                                    Some(view! {
                                        <text
                                            data-part="end-label"
                                            x=(x + if right { 10.0 } else { -10.0 }).clamp(6.0, l.width - 6.0)
                                            y=(y - 9.0).clamp(12.0, l.height - 6.0)
                                            text-anchor=if right { "start" } else { "end" }
                                        >
                                            {text}
                                        </text>
                                    })
                                })
                                .collect_view();
                            view! {
                                <g
                                    data-part="series"
                                    data-index=i
                                    data-active=move || (highlight() == Some(i)).then_some("")
                                    style=format!("--c: {};", colour(i))
                                >
                                    <path data-part="line" pathLength="1" d=t.path.clone()></path>
                                    {dots}
                                    {ends}
                                </g>
                            }
                        })
                        .collect_view();
                    view! {
                        <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) aria-hidden="true">
                            <line data-part="axis" x1=l.centre.0 - l.radius y1=l.centre.1 x2=l.centre.0 + l.radius y2=l.centre.1></line>
                            {grid}
                            {arcs}
                            <circle data-part="boundary" cx=l.centre.0 cy=l.centre.1 r=l.radius></circle>
                            {traces}
                        </svg>
                    }
                }}
                {move || {
                    let at = active.get()?;
                    let (name, s, (x, y)) = sample_at(at)?;
                    let (width, height) = layout.with(|l| (l.width, l.height));
                    let (gr, gi) = smith::gamma(s.real, s.imaginary, if z0.is_finite() && z0 > 0.0 { z0 } else { 1.0 });
                    let mut rows = Vec::new();
                    if let Some(l) = s.label.clone() {
                        rows.push(TipRow::new(None, "Sample", l));
                    }
                    rows.push(TipRow::new(None, "Impedance", impedance(&s, fmt)));
                    rows.push(TipRow::new(None, "|Γ|", format!("{:.2}", gr.hypot(gi))));
                    let tip = Tip { x: x / width, y: 1.0 - y / height, title: name, rows };
                    let side = if x > width / 2.0 { "left" } else { "right" };
                    Some(view! { <ChartTip tip swatch=colour(at.0) anchored=true side /> })
                }}
            </div>
            <Show when=move || layout.with(|l| l.traces.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .traces
                            .into_iter()
                            .enumerate()
                            .map(|(i, t)| {
                                view! {
                                    <li
                                        data-part="legend-item"
                                        data-index=i
                                        data-active=move || (highlight() == Some(i)).then_some("")
                                        on:pointerenter=move |_| hovered_trace.set(Some(i))
                                        on:pointerleave=move |_| hovered_trace.set(None)
                                    >
                                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                        {t.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr><th scope="col">"Trace"</th><th scope="col">"Sample"</th><th scope="col">"Impedance"</th></tr>
                </thead>
                <tbody>
                    {move || {
                        samples
                            .get()
                            .into_iter()
                            .map(|s| view! { <tr><th scope="row">{s.trace.clone()}</th><td>{s.label.clone().unwrap_or_default()}</td><td>{impedance(&s, fmt)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
