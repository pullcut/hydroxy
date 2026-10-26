//! `<ScatterChart>`: numeric x and y, with optional bubble size and one colour per group.
//! Geometry comes from `oxide_charts::scatter`; dots are positioned with CSS percentages.

use leptos::prelude::*;
use oxide_charts::{format_value, scatter};

pub use oxide_charts::scatter::{Group, Point};

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Dots on two numeric axes. Hover or touch to read the nearest dot; focus the plot and
/// use the arrow keys to step through dots from left to right, Home and End to jump,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn ScatterChart(
    /// The groups, one colour each in categorical order. Up to six read well.
    /// Give points a size to draw bubbles.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Interviews against satisfaction". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Caption for the x axis.
    #[prop(optional, into)]
    x_label: Option<String>,
    /// Caption for the y axis.
    #[prop(optional, into)]
    y_label: Option<String>,
    /// Name of the bubble size measure, shown in the tooltip and table.
    #[prop(default = "Size".into(), into)]
    size_label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 280)]
    height: u16,
    /// Formats x values for the axis, tooltip and table.
    #[prop(optional, into)]
    format_x: Option<Callback<f64, String>>,
    /// Formats y values for the axis, tooltip and table.
    #[prop(optional, into)]
    format_y: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fx = move |v: f64| format_x.map_or_else(|| format_value(v), |f| f.run(v));
    let fy = move |v: f64| format_y.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| groups.with(|g| scatter::layout(g, 6)));
    let active = RwSignal::new(None::<usize>);
    let point_of = move |dot: scatter::Dot| groups.with(|g| g.get(dot.group).and_then(|g| g.points.get(dot.index).copied()));
    let size_label_text = size_label.clone();
    let describe = move |i: usize| {
        let dot = layout.with(|l| l.dots.get(i).copied())?;
        let point = point_of(dot)?;
        let name = groups.with(|g| g[dot.group].name.clone());
        let mut text = format!("{name}: {} by {}", fx(point.x), fy(point.y));
        if let Some(s) = point.size {
            text = format!("{text}, {size_label_text} {}", format_value(s));
        }
        Some(text)
    };
    let move_to = move |ev: leptos::ev::PointerEvent| {
        let Some(el) = ev.current_target().and_then(|t| leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(t).ok()) else {
            return;
        };
        let (w, h) = (f64::from(el.offset_width().max(1)), f64::from(el.offset_height().max(1)));
        let (x, y) = (f64::from(ev.offset_x()) / w, 1.0 - f64::from(ev.offset_y()) / h);
        let near = layout.with(|l| scatter::nearest(&l.dots, x, y, 0.08));
        if active.get_untracked() != near {
            active.set(near);
        }
    };
    let has_sizes = move || groups.with(|g| g.iter().flat_map(|g| &g.points).any(|p| p.size.is_some()));
    let size_label_table = size_label.clone();
    let x_label_title = x_label.clone();
    let y_label_title = y_label.clone();

    view! {
        <figure data-scope="scatter-chart" data-part="root">
            {y_label_title.map(|t| view! { <div data-part="axis-title" data-axis="y" aria-hidden="true">{t}</div> })}
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .y_ticks
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div data-part="gridline" data-axis="y" style=format!("bottom: {};", pct(g.fraction))>
                                        <span data-part="tick">{fy(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        layout
                            .get()
                            .x_ticks
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div data-part="gridline" data-axis="x" style=format!("left: {};", pct(g.fraction))>
                                        <span data-part="tick">{fx(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div
                    data-part="overlay"
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to step through points.")
                    on:pointermove=move_to
                    on:pointerdown=move_to
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = layout.with(|l| scatter::step_key(&ev.key(), active.get_untracked(), &l.order)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        layout
                            .get()
                            .dots
                            .into_iter()
                            .enumerate()
                            .map(|(i, d)| {
                                let diameter = d.size.map_or(10.0, |s| 8.0 + s * 32.0);
                                view! {
                                    <span
                                        data-part="dot"
                                        data-group=d.group
                                        data-bubble=d.size.map(|_| "")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!(
                                            "left: {}; bottom: {}; --d: {diameter:.1}px; --i: {}; background: {};",
                                            pct(d.x),
                                            pct(d.y),
                                            i.min(40),
                                            colour(d.group),
                                        )
                                    ></span>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let i = active.get()?;
                        let d = layout.with(|l| l.dots.get(i).copied())?;
                        let p = point_of(d)?;
                        let name = groups.with(|g| g[d.group].name.clone());
                        let size_row = p
                            .size
                            .map(|s| {
                                view! {
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">{size_label.clone()}</span>
                                        <span data-part="tooltip-value">{format_value(s)}</span>
                                    </li>
                                }
                            });
                        Some(
                            view! {
                                <div
                                    data-part="tooltip"
                                    data-side={if d.x > 0.5 { "left" } else { "right" }}
                                    data-vertical={if d.y > 0.5 { "below" } else { "above" }}
                                    style=format!("left: {}; bottom: {};", pct(d.x), pct(d.y))
                                    aria-hidden="true"
                                >
                                    <div data-part="tooltip-label">
                                        <span data-part="swatch" style=format!("background: {};", colour(d.group))></span>
                                        {name}
                                    </div>
                                    <ul>
                                        <li data-part="tooltip-row">
                                            <span data-part="tooltip-name">{x_label.clone().unwrap_or_else(|| "x".into())}</span>
                                            <span data-part="tooltip-value">{fx(p.x)}</span>
                                        </li>
                                        <li data-part="tooltip-row">
                                            <span data-part="tooltip-name">{y_label.clone().unwrap_or_else(|| "y".into())}</span>
                                            <span data-part="tooltip-value">{fy(p.y)}</span>
                                        </li>
                                        {size_row}
                                    </ul>
                                </div>
                            },
                        )
                    }}
                </div>
            </div>
            {x_label_title.map(|t| view! { <div data-part="axis-title" data-axis="x" aria-hidden="true">{t}</div> })}
            <Show when=move || groups.with(|g| g.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        groups
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, g)| {
                                view! {
                                    <li data-part="legend-item" data-index=i>
                                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                        {g.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(&describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Group"</th>
                        <th scope="col">"X"</th>
                        <th scope="col">"Y"</th>
                        {move || has_sizes().then(|| view! { <th scope="col">{size_label_table.clone()}</th> })}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let sized = has_sizes();
                        groups
                            .get()
                            .into_iter()
                            .flat_map(|g| {
                                g.points
                                    .into_iter()
                                    .map(|p| (g.name.clone(), p))
                                    .collect::<Vec<_>>()
                            })
                            .map(|(name, p)| {
                                view! {
                                    <tr>
                                        <th scope="row">{name}</th>
                                        <td>{fx(p.x)}</td>
                                        <td>{fy(p.y)}</td>
                                        {sized.then(|| view! { <td>{p.size.map(format_value).unwrap_or_default()}</td> })}
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
