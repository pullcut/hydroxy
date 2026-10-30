//! `<NetworkChart>`: nodes and links in a force-directed layout. The layout is a fixed
//! number of seeded iterations from `hydroxy_charts::network`, so it never changes
//! between renders. Hover, touch or focus a node to pick out its neighbours.

use hydroxy_charts::arc_diagram::colour;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::{self, HINT_NODES, hinted};
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, network};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::ChartTip;
use crate::rank_kit::{GroupLegend, distinct, group_name};

pub use hydroxy_charts::network::{Link, Node};

/// The ids linked to node `i`.
fn linked(l: &network::Layout, i: usize) -> Vec<String> {
    network::neighbours(l, i)
        .iter()
        .enumerate()
        .filter(|(j, on)| **on && *j != i)
        .map(|(j, _)| l.nodes[j].id.clone())
        .collect()
}

/// A node-link diagram. Hover or touch a node to read it; the node and the nodes it
/// links to stay lit and the rest fade. Focus the chart and use the arrow keys to step
/// through the nodes, Home and End to jump, Escape to clear.
#[component]
pub fn NetworkChart(
    /// The nodes. Ids must be unique; they are also the labels.
    #[prop(into)]
    nodes: Signal<Vec<Node>>,
    /// The links, by node id. Links to unknown ids are ignored.
    #[prop(into)]
    links: Signal<Vec<Link>>,
    /// What the chart shows, such as "Who works with whom". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Names for the groups, by group number, for the legend, tooltip and table.
    /// Default: "Group 1", "Group 2" and so on.
    #[prop(optional, into)]
    groups: Vec<String>,
    /// Drawing width in px; the chart scales down to fit its container.
    #[prop(default = 640)]
    width: u16,
    /// Drawing height in px.
    #[prop(default = 420)]
    height: u16,
    /// Layout iterations. More settles tighter; the result is always the same for the same input.
    #[prop(default = 300)]
    iterations: u16,
    /// Seed for the starting positions. Change it to try another arrangement.
    #[prop(default = 1)]
    seed: u32,
    /// Formats link weights for the tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called with the node under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the node when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    let layout = Memo::new(move |_| {
        nodes.with(|n| links.with(|l| network::layout(n, l, box_w, box_h, usize::from(iterations), u64::from(seed))))
    });
    let names = StoredValue::new(groups);
    let gname = move |g: usize| names.with_value(|n| group_name(n, g));
    let active = RwSignal::new(None::<usize>);
    // `series` is the node's group and `index` the node; the value is its link weight.
    let datum = Signal::derive(move || {
        let i = active.get()?;
        layout.with(|l| {
            let node = l.nodes.get(i)?;
            Some(EventDatum {
                series: node.group,
                index: i,
                label: Some(node.id.clone()),
                value: network::weight(l, i),
                node: None,
            })
        })
    });
    watch_hover(Some(datum), on_hover);
    let near = Memo::new(move |_| active.get().map(|i| layout.with(|l| network::neighbours(l, i))));
    let node_lit =
        move |i: usize| near.with(|n| n.as_ref().is_some_and(|n| n.get(i).copied().unwrap_or(false))).then_some("");
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let describe = move |i: usize| {
        layout.with(|l| {
            let node = l.nodes.get(i)?;
            Some(format!(
                "{}, {}: {} links, weight {}",
                node.id,
                gname(node.group),
                node.degree,
                fmt(network::weight(l, i))
            ))
        })
    };

    let group_ids =
        Signal::derive(move || distinct(layout.with(|l| l.nodes.iter().map(|n| n.group).collect::<Vec<_>>())));
    let group_label = hinted(&label, HINT_NODES);
    let caption = label;
    view! {
        <figure data-scope="network-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:click=move |_| select_active(Some(datum), on_select)
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), layout.with(|l| l.nodes.len())) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {width} {height}") aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        let max = l.edges.iter().map(|e| e.value).fold(1.0, f64::max);
                        let edges = l
                            .edges
                            .iter()
                            .map(|e| {
                                let (from, to) = (&l.nodes[e.source], &l.nodes[e.target]);
                                let (src, dst) = (e.source, e.target);
                                view! {
                                    <line
                                        data-part="link"
                                        pathLength="1"
                                        data-highlighted=move || (active.get() == Some(src) || active.get() == Some(dst)).then_some("")
                                        x1=from.x
                                        y1=from.y
                                        x2=to.x
                                        y2=to.y
                                        stroke-width=1.0 + 2.0 * e.value / max
                                    ></line>
                                }
                            })
                            .collect_view();
                        let dots = l
                            .nodes
                            .iter()
                            .enumerate()
                            .map(|(i, node)| {
                                view! {
                                    <g
                                        data-part="node"
                                        data-group=node.group
                                        data-highlighted=move || node_lit(i)
                                        data-selected=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {i}; --ox-n: {};", l.nodes.len())
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerdown=move |_| pick(i)
                                    >
                                        <circle cx=node.x cy=node.y r=node.radius fill=colour(node.group)></circle>
                                        <text data-part="node-label" x=node.x y=node.y + node.radius + 12.0 text-anchor="middle">
                                            {node.id.clone()}
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="links">{edges}</g>
                            <g data-part="nodes">{dots}</g>
                        }
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let (node, total) = layout.with(|l| l.nodes.get(i).cloned().map(|n| (n, network::weight(l, i))))?;
                    let (fx, fy) = (node.x / box_w, node.y / box_h);
                    let tip = Tip {
                        x: fx,
                        y: 1.0 - fy,
                        title: node.id,
                        rows: vec![
                            TipRow::new(None, "Group", gname(node.group)),
                            TipRow::new(None, "Links", node.degree.to_string()),
                            TipRow::new(None, "Weight", fmt(total)),
                        ],
                    };
                    Some(view! { <ChartTip tip swatch=colour(node.group) anchored=true /> })
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
                        l.nodes
                            .iter()
                            .enumerate()
                            .map(|(i, node)| {
                                view! {
                                    <tr>
                                        <th scope="row">{node.id.clone()}</th>
                                        <td>{gname(node.group)}</td>
                                        <td>{node.degree}</td>
                                        <td>{fmt(network::weight(&l, i))}</td>
                                        <td>{linked(&l, i).join(", ")}</td>
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
