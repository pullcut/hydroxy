//! `<ArcDiagramChart>`: nodes in a row, each link a semicircle above the line. Ordering
//! by group keeps a cluster's arcs short, so links between clusters stand out as the
//! wide ones. Geometry comes from `hydroxy_charts::arc_diagram`.

use hydroxy_charts::arc_diagram::{self, colour};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::format_value;
use hydroxy_charts::nav::{self, HINT_NODES, hinted};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::rank_kit::{GroupLegend, distinct, group_name};

pub use hydroxy_charts::arc_diagram::{Link, Node, Order};

/// An arc diagram. Hover, touch or focus a node to light its arcs and the nodes at their
/// other ends. Focus the chart and use the arrow keys to step through the nodes from left
/// to right, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn ArcDiagramChart(
    /// The nodes. Ids must be unique; they are also the labels.
    #[prop(into)]
    nodes: Signal<Vec<Node>>,
    /// The links, by node id. Links to unknown ids and self-links are ignored.
    #[prop(into)]
    links: Signal<Vec<Link>>,
    /// What the chart shows, such as "Characters who share a scene". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names for the groups, by group number, for the legend, tooltip and table.
    /// Default: "Group 1", "Group 2" and so on.
    #[prop(optional, into)]
    groups: Vec<String>,
    /// How nodes are ordered along the line. Default: by group, most-linked first.
    #[prop(optional)]
    order: Order,
    /// Arc width follows link value. Turn off when values are all alike.
    #[prop(default = true)]
    weighted: bool,
    /// Drawing width in px; the chart scales down to fit its container.
    #[prop(default = 720)]
    width: u16,
    /// Formats link values for the tooltip and table. Default: whole numbers, else one decimal place.
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
    let layout = Memo::new(move |_| {
        nodes.with(|n| links.with(|l| arc_diagram::layout(n, l, f64::from(width), order, weighted)))
    });
    let names = StoredValue::new(groups);
    let gname = move |g: usize| names.with_value(|n| group_name(n, g));
    let active = RwSignal::new(None::<usize>);
    let near = Memo::new(move |_| active.get().map(|i| layout.with(|l| arc_diagram::neighbours(l, i))));
    let node_lit =
        move |i: usize| near.with(|n| n.as_ref().is_some_and(|n| n.get(i).copied().unwrap_or(false))).then_some("");
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    // Series is the node's group, index its place in `nodes`, value its weight.
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            layout.with(|l| {
                let node = l.nodes.get(i)?;
                Some(EventDatum { series: node.group, ..EventDatum::category(i, node.id.clone(), node.weight) })
            })
        })
    });
    watch_hover(datum, on_hover);
    let describe = move |i: usize| {
        layout.with(|l| {
            let node = l.nodes.get(i)?;
            Some(format!("{}, {}: {} links, weight {}", node.id, gname(node.group), node.degree, fmt(node.weight)))
        })
    };
    // Keyboard steps left to right, by position in the line.
    let step = move |key: &str| {
        layout.with(|l| {
            let pos = active.get_untracked().and_then(|a| l.order.iter().position(|&o| o == a));
            nav::list_key(key, pos, l.order.len()).map(|next| next.map(|p| l.order[p]))
        })
    };
    let group_ids =
        Signal::derive(move || distinct(layout.with(|l| l.nodes.iter().map(|n| n.group).collect::<Vec<_>>())));

    let group_label = hinted(&label, HINT_NODES);
    let caption = label;
    view! {
        <figure data-scope="arc-diagram-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        select_active(datum, on_select);
                    }
                    if let Some(next) = step(&ev.key()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=move || layout.with(|l| format!("0 0 {} {:.0}", l.width, l.height)) aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        let arcs = l
                            .arcs
                            .iter()
                            .enumerate()
                            .map(|(k, a)| {
                                let (src, dst) = (a.source, a.target);
                                view! {
                                    <path
                                        data-part="arc"
                                        pathLength="1"
                                        data-highlighted=move || (active.get() == Some(src) || active.get() == Some(dst)).then_some("")
                                        d=a.path.clone()
                                        stroke=colour(l.nodes[src].group)
                                        stroke-width=format!("{:.2}", a.width)
                                        style=format!("--i: {};", k.min(40))
                                    ></path>
                                }
                            })
                            .collect_view();
                        let base = l.baseline;
                        let dots = l
                            .order
                            .iter()
                            .enumerate()
                            .map(|(slot, &i)| {
                                let node = &l.nodes[i];
                                view! {
                                    <g
                                        data-part="node"
                                        data-group=node.group
                                        data-highlighted=move || node_lit(i)
                                        data-selected=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {};", slot.min(40))
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerdown=move |_| pick(i)
                                        on:click=move |_| {
                                            pick(i);
                                            select_active(datum, on_select);
                                        }
                                    >
                                        <circle data-part="hit" cx=node.x cy=base r="12"></circle>
                                        <circle data-part="dot" cx=node.x cy=base r=node.radius fill=colour(node.group)></circle>
                                        <text data-part="node-label" transform=format!("translate({:.2} {:.2}) rotate(45)", node.x - 3.0, base + node.radius + 8.0)>
                                            {node.id.clone()}
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view();
                        view! {
                            <line data-part="baseline" x1=arc_diagram::PAD - 12.0 x2=l.width - arc_diagram::PAD + 12.0 y1=base y2=base></line>
                            <g data-part="arcs">{arcs}</g>
                            <g data-part="nodes">{dots}</g>
                        }
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let (node, fy, fx, others) = layout.with(|l| {
                        let n = l.nodes.get(i)?.clone();
                        Some((n.clone(), l.baseline / l.height, n.x / l.width, arc_diagram::linked(l, i)))
                    })?;
                    let shown = others.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
                    let more = others.len().saturating_sub(4);
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if fx > 0.5 { "left" } else { "right" }}
                                style=format!("left: {:.3}%; top: {:.3}%;", fx * 100.0, fy * 100.0)
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: {};", colour(node.group))></span>
                                    {node.id}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Group"</span>
                                        <span data-part="tooltip-value">{gname(node.group)}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Links"</span>
                                        <span data-part="tooltip-value">{node.degree}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Weight"</span>
                                        <span data-part="tooltip-value">{fmt(node.weight)}</span>
                                    </li>
                                </ul>
                                {(!others.is_empty()).then(|| view! {
                                    <p data-part="tooltip-note">
                                        {shown}
                                        {(more > 0).then(|| format!(" and {more} more"))}
                                    </p>
                                })}
                            </div>
                        },
                    )
                }}
            </div>
            <GroupLegend groups=group_ids name=Callback::new(gname) colour=colour />
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Node"</th>
                        <th scope="col">"Group"</th>
                        <th scope="col">"Links"</th>
                        <th scope="col">"Weight"</th>
                        <th scope="col">"Linked to"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        l.order
                            .iter()
                            .map(|&i| {
                                let node = &l.nodes[i];
                                view! {
                                    <tr>
                                        <th scope="row">{node.id.clone()}</th>
                                        <td>{gname(node.group)}</td>
                                        <td>{node.degree}</td>
                                        <td>{fmt(node.weight)}</td>
                                        <td>{arc_diagram::linked(&l, i).join(", ")}</td>
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
