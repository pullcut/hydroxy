//! `NetworkChart`: nodes and links in a force-directed layout. The layout is a fixed
//! number of seeded iterations from `hydroxy_charts::network`, so it never changes
//! between renders. Hover, touch or focus a node to pick out its neighbours.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, nav, network};

pub use hydroxy_charts::network::{Link, Node};

fn colour(group: usize) -> String {
    format!("var(--chart-{})", group % 6 + 1)
}

fn group_name(names: &[String], group: usize) -> String {
    names.get(group).cloned().unwrap_or_else(|| format!("Group {}", group + 1))
}

/// The ids linked to node `i`.
fn linked(l: &network::Layout, i: usize) -> Vec<String> {
    network::neighbours(l, i).iter().enumerate().filter(|(j, on)| **on && *j != i).map(|(j, _)| l.nodes[j].id.clone()).collect()
}

/// A node-link diagram. Hover or touch a node to read it; the node and the nodes it
/// links to stay lit and the rest fade. Focus the chart and use the arrow keys to step
/// through the nodes, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn NetworkChart(
    /// The nodes. Ids must be unique; they are also the labels.
    nodes: ReadSignal<Vec<Node>>,
    /// The links, by node id. Links to unknown ids are ignored.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Who works with whom". Used as the accessible name.
    label: String,
    /// Names for the groups, by group number, for the legend, tooltip and table.
    /// Default: "Group 1", "Group 2" and so on.
    #[props(default)]
    groups: Vec<String>,
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
    /// Formats link weights for the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    let mut active = use_signal(|| None::<usize>);
    let lay = network::layout(&nodes(), &links(), box_w, box_h, usize::from(iterations), u64::from(seed));
    let n = lay.nodes.len();
    let current = active().filter(|&i| i < n);
    let near = current.map(|i| network::neighbours(&lay, i));
    let max = lay.edges.iter().map(|e| e.value).fold(1.0, f64::max);
    let mut group_ids: Vec<usize> = lay.nodes.iter().map(|n| n.group).collect();
    group_ids.sort_unstable();
    group_ids.dedup();
    let gname = |g: usize| group_name(&groups, g);
    let tip = current.map(|i| (lay.nodes[i].clone(), network::weight(&lay, i)));
    let announcement = tip.as_ref().map(|(node, total)| format!("{}, {}: {} links, weight {}", node.id, gname(node.group), node.degree, fmt(*total)));

    rsx! {
        figure {
            "data-scope": "network-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the nodes.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 {width} {height}", "aria-hidden": "true",
                    g { "data-part": "links",
                        for (k, e) in lay.edges.iter().enumerate() {
                            line {
                                key: "{k}",
                                "data-part": "link",
                                "pathLength": "1",
                                "data-highlighted": (current == Some(e.source) || current == Some(e.target)).then_some(""),
                                x1: "{lay.nodes[e.source].x}",
                                y1: "{lay.nodes[e.source].y}",
                                x2: "{lay.nodes[e.target].x}",
                                y2: "{lay.nodes[e.target].y}",
                                stroke_width: "{1.0 + 2.0 * e.value / max}",
                            }
                        }
                    }
                    g { "data-part": "nodes",
                        for (i, node) in lay.nodes.iter().enumerate() {
                            g {
                                key: "{node.id}",
                                "data-part": "node",
                                "data-group": "{node.group}",
                                "data-highlighted": near.as_ref().is_some_and(|x| x.get(i).copied().unwrap_or(false)).then_some(""),
                                "data-selected": (current == Some(i)).then_some(""),
                                style: "--i: {i.min(40)};",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                circle { cx: "{node.x}", cy: "{node.y}", r: "{node.radius}", fill: colour(node.group) }
                                text { "data-part": "node-label", x: "{node.x}", y: "{node.y + node.radius + 12.0}", text_anchor: "middle", "{node.id}" }
                            }
                        }
                    }
                }
                if let Some((node, total)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if node.x / box_w > 0.5 { "left" } else { "right" },
                        style: "left: {node.x / box_w * 100.0:.3}%; top: {node.y / box_h * 100.0:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour(node.group)};" }
                            "{node.id}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Group" }
                                span { "data-part": "tooltip-value", {gname(node.group)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Links" }
                                span { "data-part": "tooltip-value", "{node.degree}" }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Weight" }
                                span { "data-part": "tooltip-value", {fmt(total)} }
                            }
                        }
                    }
                }
            }
            if group_ids.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for g in group_ids.iter().copied() {
                        li { "data-part": "legend-item", "data-group": "{g}",
                            span { "data-part": "swatch", style: "background: {colour(g)};" }
                            {gname(g)}
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Node" }
                        th { scope: "col", "Group" }
                        th { scope: "col", "Links" }
                        th { scope: "col", "Weight" }
                        th { scope: "col", "Linked to" }
                    }
                }
                tbody {
                    for (i, node) in lay.nodes.iter().enumerate() {
                        tr {
                            th { scope: "row", "{node.id}" }
                            td { {gname(node.group)} }
                            td { "{node.degree}" }
                            td { {fmt(network::weight(&lay, i))} }
                            td { {linked(&lay, i).join(", ")} }
                        }
                    }
                }
            }
        }
    }
}
