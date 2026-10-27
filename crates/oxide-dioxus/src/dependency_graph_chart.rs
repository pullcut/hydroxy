//! `DependencyGraphChart`: a directed graph in layers, drawn left to right. The layered
//! layout comes from `oxide_charts::dag`. Hover or focus a node to trace everything
//! upstream and downstream of it.

use dioxus::prelude::*;
use oxide_charts::{dag, format_value, nav};

pub use oxide_charts::network::{Link, Node};

fn colour(group: usize) -> String {
    if group < 6 { format!("var(--chart-{})", group + 1) } else { "var(--chart-muted)".to_string() }
}

/// A layered dependency graph. Focus the chart and use the arrow keys to step through
/// nodes column by column, Home and End to jump, Escape to clear. The active node's
/// upstream and downstream stay lit and the rest fade.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn DependencyGraphChart(
    /// The nodes. Ids must be unique; they are also the labels. Groups set the colour, up to six.
    nodes: ReadSignal<Vec<Node>>,
    /// The links, source → target: the source feeds the target. Unknown ids and self-links are ignored.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Build pipeline". Used as the accessible name.
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[props(default = 720)]
    width: u16,
    /// Drawing height in px.
    #[props(default = 360)]
    height: u16,
    /// Legend names for the groups, by group number. Missing names read "Group N".
    #[props(default)]
    group_names: Vec<String>,
    /// Width of each node box in px.
    #[props(default = 112)]
    node_width: u16,
    /// Formats link weights in the tooltip and readout.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (w, h, nw) = (f64::from(width), f64::from(height), f64::from(node_width));
    let nodes_v = nodes();
    let lay = dag::layout(&nodes_v, &links(), w, h, nw);
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < lay.nodes.len());
    let reach = current.map(|i| dag::reach(&lay, i));
    let position = current.and_then(|i| lay.order.iter().position(|&o| o == i));
    let node_lit = |i: usize| reach.as_ref().map(|(up, down)| current == Some(i) || up[i] || down[i]);
    let edge_lit = |s: usize, t: usize| reach.as_ref().map(|(up, down)| (down[t] && (current == Some(s) || down[s])) || (up[s] && (current == Some(t) || up[t])));
    let counts = reach.as_ref().map(|(up, down)| (up.iter().filter(|x| **x).count(), down.iter().filter(|x| **x).count())).unwrap_or_default();
    let tip = current.map(|i| {
        let node = lay.nodes[i].clone();
        let (inn, out) = lay.edges.iter().fold((0.0, 0.0), |(inn, out), e| (inn + if e.target == i { e.value } else { 0.0 }, out + if e.source == i { e.value } else { 0.0 }));
        let rows = vec![
            ("Layer", (node.layer + 1).to_string()),
            ("Upstream", counts.0.to_string()),
            ("Downstream", counts.1.to_string()),
            ("Flow in", fmt(inn)),
            ("Flow out", fmt(out)),
        ];
        (node.x / w * 100.0, (node.y - 14.0) / h * 100.0, node, rows)
    });
    let announcement =
        tip.as_ref().map(|(_, _, node, _)| format!("{}: layer {}, {} upstream, {} downstream", node.id, node.layer + 1, counts.0, counts.1)).unwrap_or_default();
    let mut groups: Vec<usize> = nodes_v.iter().map(|n| n.group.min(6)).collect();
    groups.sort_unstable();
    groups.dedup();
    let names = |pick: &dyn Fn(&dag::DagEdge) -> Option<usize>| lay.edges.iter().filter_map(pick).map(|j| lay.nodes[j].id.clone()).collect::<Vec<_>>().join(", ");
    let rows: Vec<(String, usize, String, String)> = lay
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), n.layer + 1, names(&|e| (e.target == i).then_some(e.source)), names(&|e| (e.source == i).then_some(e.target))))
        .collect();
    let group_name = |g: usize| if g < 6 { group_names.get(g).cloned().unwrap_or_else(|| format!("Group {}", g + 1)) } else { "Other".to_string() };
    let order = lay.order.clone();
    let half = nw / 2.0;

    rsx! {
        figure {
            "data-scope": "dependency-graph-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {width} {height}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through nodes.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::list_key(&e.key().to_string(), position, order.len()) {
                            e.prevent_default();
                            active.set(next.map(|k| order[k]));
                        }
                    },
                    g { "data-part": "links",
                        for (k , e) in lay.edges.iter().enumerate() {
                            path {
                                key: "{k}",
                                "data-part": "link",
                                "data-reversed": e.reversed.then_some(""),
                                "pathLength": (!e.reversed).then_some("1"),
                                "data-lit": (edge_lit(e.source, e.target) == Some(true)).then_some(""),
                                "data-dim": (edge_lit(e.source, e.target) == Some(false)).then_some(""),
                                d: "{e.path}",
                                style: "--i: {k.min(40)};",
                            }
                        }
                    }
                    g { "data-part": "nodes",
                        for (i , n) in lay.nodes.iter().enumerate() {
                            g {
                                key: "{n.id}",
                                "data-part": "node",
                                "data-active": (current == Some(i)).then_some(""),
                                "data-lit": (node_lit(i) == Some(true)).then_some(""),
                                "data-dim": (node_lit(i) == Some(false)).then_some(""),
                                style: "--c: {colour(n.group)}; --i: {n.layer};",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                rect {
                                    x: "{n.x - half:.1}",
                                    y: "{n.y - 14.0:.1}",
                                    width: "{nw}",
                                    height: "28",
                                    rx: "6",
                                }
                                text {
                                    "data-part": "label",
                                    x: "{n.x:.1}",
                                    y: "{n.y:.1}",
                                    text_anchor: "middle",
                                    dominant_baseline: "central",
                                    "{n.id}"
                                }
                            }
                        }
                    }
                }
                if let Some((left , top , node , rows)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if left > 50.0 { "left" } else { "right" },
                        "data-vertical": if top < 25.0 { "below" } else { "above" },
                        style: "left: {left:.2}%; top: {top:.2}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour(node.group)};" }
                            "{node.id}"
                        }
                        ul {
                            for (k , v) in rows {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "{k}" }
                                    span { "data-part": "tooltip-value", "{v}" }
                                }
                            }
                        }
                    }
                }
            }
            if groups.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for g in groups {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {colour(g)};" }
                            "{group_name(g)}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Node" }
                        th { scope: "col", "Layer" }
                        th { scope: "col", "Comes from" }
                        th { scope: "col", "Feeds into" }
                    }
                }
                tbody {
                    for (id , layer , from , into) in rows {
                        tr {
                            th { scope: "row", "{id}" }
                            td { "{layer}" }
                            td { "{from}" }
                            td { "{into}" }
                        }
                    }
                }
            }
        }
    }
}
