//! `<NetworkChart>`: nodes and links in a force-directed layout. The layout is a fixed
//! number of seeded iterations from `oxide_charts::network`, so it never changes
//! between renders. Hover, touch or focus a node to pick out its neighbours.

use leptos::prelude::*;
use oxide_charts::{format_value, nav, network};

pub use oxide_charts::network::{Link, Node};

fn colour(group: usize) -> String {
    format!("var(--chart-{})", group % 6 + 1)
}

fn group_name(names: &[String], group: usize) -> String {
    names.get(group).cloned().unwrap_or_else(|| format!("Group {}", group + 1))
}

/// The distinct groups, in ascending order.
fn groups_of(l: &network::Layout) -> Vec<usize> {
    let mut g: Vec<usize> = l.nodes.iter().map(|n| n.group).collect();
    g.sort_unstable();
    g.dedup();
    g
}

/// The ids linked to node `i`.
fn linked(l: &network::Layout, i: usize) -> Vec<String> {
    network::neighbours(l, i).iter().enumerate().filter(|(j, on)| **on && *j != i).map(|(j, _)| l.nodes[j].id.clone()).collect()
}

/// A node-link diagram. Hover or touch a node to read it; the node and the nodes it
/// links to stay lit and the rest fade. Focus the chart and use the arrow keys to step
/// through the nodes, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
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
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    let layout = Memo::new(move |_| {
        nodes.with(|n| links.with(|l| network::layout(n, l, box_w, box_h, usize::from(iterations), u64::from(seed))))
    });
    let names = StoredValue::new(groups);
    let gname = move |g: usize| names.with_value(|n| group_name(n, g));
    let active = RwSignal::new(None::<usize>);
    let near = Memo::new(move |_| active.get().map(|i| layout.with(|l| network::neighbours(l, i))));
    let node_lit = move |i: usize| near.with(|n| n.as_ref().is_some_and(|n| n.get(i).copied().unwrap_or(false))).then_some("");
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let describe = move |i: usize| {
        layout.with(|l| {
            let node = l.nodes.get(i)?;
            Some(format!("{}, {}: {} links, weight {}", node.id, gname(node.group), node.degree, fmt(network::weight(l, i))))
        })
    };

    let group_label = format!("{label}. Use the arrow keys to step through the nodes.");
    let caption = label;
    view! {
        <figure data-scope="network-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
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
                                        style=format!("--i: {};", i.min(40))
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
                                        <span data-part="tooltip-value">{fmt(total)}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <Show when=move || layout.with(|l| groups_of(l).len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        layout
                            .with(groups_of)
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <li data-part="legend-item" data-group=g>
                                        <span data-part="swatch" style=format!("background: {};", colour(g))></span>
                                        {gname(g)}
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
