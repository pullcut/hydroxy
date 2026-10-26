//! `<NetworkChart>`: nodes and links in a force-directed layout. The layout is a fixed
//! number of seeded iterations from `oxide_charts::network`, so it never changes
//! between renders. Hover or focus a node to pick out its neighbours.

use leptos::prelude::*;
use oxide_charts::network;

pub use oxide_charts::network::{Link, Node};

fn colour(group: usize) -> String {
    format!("var(--chart-{})", group % 6 + 1)
}

/// A node-link diagram. Tab through the nodes; the focused node and the nodes it links
/// to stay lit and the rest fade.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
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
) -> impl IntoView {
    let layout = Memo::new(move |_| {
        nodes.with(|n| links.with(|l| network::layout(n, l, f64::from(width), f64::from(height), usize::from(iterations), u64::from(seed))))
    });
    let active = RwSignal::new(None::<usize>);
    let near = Memo::new(move |_| active.get().map(|i| layout.with(|l| network::neighbours(l, i))));
    let node_lit = move |i: usize| near.with(|n| n.as_ref().is_some_and(|n| n.get(i).copied().unwrap_or(false))).then_some("");

    let caption = label.clone();
    view! {
        <figure data-scope="network-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <svg viewBox=format!("0 0 {width} {height}") role="group" aria-label=label>
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
                        .map(|(i, n)| {
                            view! {
                                <g
                                    data-part="node"
                                    data-highlighted=move || node_lit(i)
                                    data-selected=move || (active.get() == Some(i)).then_some("")
                                    tabindex="0"
                                    role="img"
                                    aria-label=format!("{}, {} links", n.id, n.degree)
                                    on:pointerenter=move |_| active.set(Some(i))
                                    on:pointerleave=move |_| active.set(None)
                                    on:focus=move |_| active.set(Some(i))
                                    on:blur=move |_| active.set(None)
                                >
                                    <circle cx=n.x cy=n.y r=n.radius fill=colour(n.group)></circle>
                                    <text data-part="node-label" x=n.x y=n.y + n.radius + 12.0 text-anchor="middle">
                                        {n.id.clone()}
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
            <ul data-part="sr-table" aria-label=format!("{caption}: nodes and their links")>
                {move || {
                    let l = layout.get();
                    (0..l.nodes.len())
                        .map(|i| {
                            let linked: Vec<&str> = network::neighbours(&l, i)
                                .iter()
                                .enumerate()
                                .filter(|(j, on)| **on && *j != i)
                                .map(|(j, _)| l.nodes[j].id.as_str())
                                .collect();
                            view! { <li>{format!("{}: linked to {}", l.nodes[i].id, if linked.is_empty() { "nothing".into() } else { linked.join(", ") })}</li> }
                        })
                        .collect_view()
                }}
            </ul>
        </figure>
    }
}
