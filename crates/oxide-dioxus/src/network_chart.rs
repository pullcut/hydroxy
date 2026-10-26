//! `NetworkChart`: nodes and links in a force-directed layout. The layout is a fixed
//! number of seeded iterations from `oxide_charts::network`, so it never changes
//! between renders. Hover or focus a node to pick out its neighbours.

use dioxus::prelude::*;
use oxide_charts::network;

pub use oxide_charts::network::{Link, Node};

fn colour(group: usize) -> String {
    format!("var(--chart-{})", group % 6 + 1)
}

/// A node-link diagram. Tab through the nodes; the focused node and the nodes it links
/// to stay lit and the rest fade.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn NetworkChart(
    /// The nodes. Ids must be unique; they are also the labels.
    nodes: ReadSignal<Vec<Node>>,
    /// The links, by node id. Links to unknown ids are ignored.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Who works with whom". Used as the accessible name.
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[props(default = 640)]
    width: u16,
    /// Drawing height in px.
    #[props(default = 420)]
    height: u16,
    /// Layout iterations. More settles tighter; the result is always the same for the same input.
    #[props(default = 300)]
    iterations: u16,
    /// Seed for the starting positions. Change it to try another arrangement.
    #[props(default = 1)]
    seed: u32,
) -> Element {
    let mut active = use_signal(|| None::<usize>);
    let lay = network::layout(&nodes(), &links(), f64::from(width), f64::from(height), usize::from(iterations), u64::from(seed));
    let near = active().map(|i| network::neighbours(&lay, i));
    let max = lay.edges.iter().map(|e| e.value).fold(1.0, f64::max);

    rsx! {
        figure {
            "data-scope": "network-chart",
            "data-part": "root",
            "data-active": active().map(|_| ""),
            style: "max-width: {width}px;",
            svg { view_box: "0 0 {width} {height}", role: "group", "aria-label": "{label}",
                g { "data-part": "links",
                    for (k, e) in lay.edges.iter().enumerate() {
                        line {
                            key: "{k}",
                            "data-part": "link",
                            "data-highlighted": (active() == Some(e.source) || active() == Some(e.target)).then_some(""),
                            x1: "{lay.nodes[e.source].x}",
                            y1: "{lay.nodes[e.source].y}",
                            x2: "{lay.nodes[e.target].x}",
                            y2: "{lay.nodes[e.target].y}",
                            stroke_width: "{1.0 + 2.0 * e.value / max}",
                        }
                    }
                }
                g { "data-part": "nodes",
                    for (i, n) in lay.nodes.iter().enumerate() {
                        g {
                            key: "{n.id}",
                            "data-part": "node",
                            "data-highlighted": near.as_ref().is_some_and(|x| x.get(i).copied().unwrap_or(false)).then_some(""),
                            "data-selected": (active() == Some(i)).then_some(""),
                            tabindex: "0",
                            role: "img",
                            "aria-label": format!("{}, {} links", n.id, n.degree),
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerleave: move |_| active.set(None),
                            onfocus: move |_| active.set(Some(i)),
                            onblur: move |_| active.set(None),
                            circle { cx: "{n.x}", cy: "{n.y}", r: "{n.radius}", fill: colour(n.group) }
                            text { "data-part": "node-label", x: "{n.x}", y: "{n.y + n.radius + 12.0}", text_anchor: "middle", "{n.id}" }
                        }
                    }
                }
            }
            ul { "data-part": "sr-table", "aria-label": "{label}: nodes and their links",
                for i in 0..lay.nodes.len() {
                    li {
                        {
                            let linked: Vec<&str> = network::neighbours(&lay, i)
                                .iter()
                                .enumerate()
                                .filter(|(j, on)| **on && *j != i)
                                .map(|(j, _)| lay.nodes[j].id.as_str())
                                .collect();
                            format!("{}: linked to {}", lay.nodes[i].id, if linked.is_empty() { "nothing".into() } else { linked.join(", ") })
                        }
                    }
                }
            }
        }
    }
}
