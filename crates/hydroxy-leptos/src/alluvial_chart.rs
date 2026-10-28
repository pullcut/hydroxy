//! `<AlluvialChart>`: how members move between categories across ordered steps. Each
//! step is a column of stacked categories, always in the same order; ribbons carry the
//! members from one step's category to the next's. Layout comes from
//! `hydroxy_charts::alluvial`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{alluvial, format_value, nav};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::sets_kit::SetsTip;

pub use hydroxy_charts::alluvial::Cohort;

/// Room at the top for the step names.
const HEAD: f64 = 26.0;

/// An alluvial diagram. Colour follows the category, so a ribbon shows where its
/// members came from. Hover or touch a category to light its ribbons, or a ribbon to read
/// how many moved; focus the chart and use the arrow keys to step through the categories
/// and then the ribbons, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn AlluvialChart(
    /// Groups of members that share a path, one category per step. Categories stack in
    /// order of first appearance; paths shorter than the longest are ignored.
    #[prop(into)]
    data: Signal<Vec<Cohort>>,
    /// Step names, such as months, in order.
    #[prop(into)]
    steps: Signal<Vec<String>>,
    /// What the chart shows, such as "Customers by plan, quarter to quarter". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[prop(default = 720)]
    width: u16,
    /// Drawing height in px.
    #[prop(default = 360)]
    height: u16,
    /// Formats counts for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
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
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    let layout = Memo::new(move |_| data.with(|d| alluvial::layout(d, box_w, box_h, HEAD, 12.0, 10.0)));
    // Marks are the nodes first, then the ribbons.
    let active = RwSignal::new(None::<usize>);
    let lit = Memo::new(move |_| active.get().map(|m| layout.with(|l| l.lit(m))));
    let node_lit =
        move |i: usize| lit.with(|x| x.as_ref().is_some_and(|(n, _)| n.get(i).copied().unwrap_or(false))).then_some("");
    let ribbon_lit =
        move |k: usize| lit.with(|x| x.as_ref().is_some_and(|(_, r)| r.get(k).copied().unwrap_or(false))).then_some("");
    let pick = move |m: usize| {
        if active.get_untracked() != Some(m) {
            active.set(Some(m));
        }
    };
    // A node reports its step and category; a ribbon reports its source step and its own index.
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let idx = active.get()?;
            layout.with(|lay| {
                let count = lay.nodes.len();
                if let Some(node) = lay.nodes.get(idx) {
                    let name = lay.categories[node.category].clone();
                    return Some(EventDatum {
                        series: node.step,
                        ..EventDatum::category(node.category, name, node.value)
                    });
                }
                let rib = lay.ribbons.get(idx.checked_sub(count)?)?;
                let (src, dst) = (&lay.nodes[rib.source], &lay.nodes[rib.target]);
                let name = format!("{} → {}", lay.categories[src.category], lay.categories[dst.category]);
                Some(EventDatum { series: src.step, ..EventDatum::category(idx - count, name, rib.value) })
            })
        })
    });
    watch_hover(datum, on_hover);
    let tip = move || active.get().and_then(|m| steps.with(|s| layout.with(|l| l.tip(m, s, box_w, box_h, &fmt))));
    let step_name = move |t: usize| steps.with(|s| s.get(t).cloned().unwrap_or_else(|| format!("Step {}", t + 1)));
    let group_label = format!("{label}. Use the arrow keys to step through the categories and then the ribbons.");
    let caption = label;

    view! {
        <figure data-scope="alluvial-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=format!("0 0 {width} {height}")
                    tabindex="0"
                    role="group"
                    aria-label=group_label
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            select_active(datum, on_select);
                        }
                        let len = layout.with(|l| l.nodes.len() + l.ribbons.len());
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), len) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let n = l.nodes.len();
                        let last = l.steps.saturating_sub(1);
                        let ribbons = l
                            .ribbons
                            .iter()
                            .enumerate()
                            .map(|(k, r)| {
                                view! {
                                    <path
                                        data-part="ribbon"
                                        data-highlighted=move || ribbon_lit(k)
                                        d=r.path.clone()
                                        fill=l.colour(l.nodes[r.source].category)
                                        on:pointerenter=move |_| pick(n + k)
                                        on:pointerdown=move |_| pick(n + k)
                                        on:click=move |_| {
                                            pick(n + k);
                                            select_active(datum, on_select);
                                        }
                                    ></path>
                                }
                            })
                            .collect_view();
                        let nodes = l
                            .nodes
                            .iter()
                            .enumerate()
                            .map(|(i, node)| {
                                view! {
                                    <rect
                                        data-part="node"
                                        data-highlighted=move || node_lit(i)
                                        data-selected=move || (active.get() == Some(i)).then_some("")
                                        x=node.x
                                        y=node.y
                                        width=l.node_width
                                        height=node.height.max(1.0)
                                        rx="2"
                                        fill=l.colour(node.category)
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerdown=move |_| pick(i)
                                        on:click=move |_| {
                                            pick(i);
                                            select_active(datum, on_select);
                                        }
                                    ></rect>
                                }
                            })
                            .collect_view();
                        let heads = l
                            .columns
                            .iter()
                            .enumerate()
                            .map(|(t, &x)| {
                                let (tx, anchor) = match t {
                                    0 => (x, "start"),
                                    t if t == last => (x + l.node_width, "end"),
                                    _ => (x + l.node_width / 2.0, "middle"),
                                };
                                view! { <text data-part="step" x=tx y="12" text-anchor=anchor>{step_name(t)}</text> }
                            })
                            .collect_view();
                        let labels = l
                            .nodes
                            .iter()
                            .filter(|node| node.height >= 14.0)
                            .map(|node| {
                                let at_end = node.step == last && last > 0;
                                let (tx, anchor) = if at_end { (node.x - 6.0, "end") } else { (node.x + l.node_width + 6.0, "start") };
                                view! {
                                    <text data-part="node-label" x=tx y=node.y + node.height / 2.0 text-anchor=anchor dominant-baseline="middle">
                                        {l.categories[node.category].clone()}
                                        <tspan data-part="node-value" dx="6">{fmt(node.value)}</tspan>
                                    </text>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="steps">{heads}</g>
                            <g data-part="flow">
                                <g data-part="ribbons">{ribbons}</g>
                                <g data-part="nodes">{nodes}</g>
                            </g>
                            <g data-part="labels">{labels}</g>
                        }
                    }}
                </svg>
                {move || tip().map(|tip| view! { <SetsTip tip=tip /> })}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || tip().map(|t| t.announce()).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Steps"</th>
                        <th scope="col">"From"</th>
                        <th scope="col">"To"</th>
                        <th scope="col">"Members"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        l.ribbons
                            .iter()
                            .map(|r| {
                                let (s, t) = (&l.nodes[r.source], &l.nodes[r.target]);
                                view! {
                                    <tr>
                                        <td>{format!("{} → {}", step_name(s.step), step_name(t.step))}</td>
                                        <td>{l.categories[s.category].clone()}</td>
                                        <td>{l.categories[t.category].clone()}</td>
                                        <td>{fmt(r.value)}</td>
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
