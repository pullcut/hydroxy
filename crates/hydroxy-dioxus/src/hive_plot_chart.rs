//! `HivePlotChart`: two to four axes radiating from the centre, one per node type. A
//! node sits on its type's axis, as far out as its value, so the picture depends only on
//! the data and never on a layout's whims. Geometry comes from `hydroxy_charts::hive`.

use dioxus::prelude::*;
use hydroxy_charts::arc_diagram::colour;
use hydroxy_charts::{format_value, hive, nav};

pub use hydroxy_charts::hive::{Link, Node};

fn axis_name(names: &[String], axis: usize) -> String {
    names.get(axis).cloned().unwrap_or_else(|| format!("Axis {}", axis + 1))
}

/// How many axes to draw: one per name, or one per axis the nodes use, 2 to 4.
fn axis_count(names: &[String], nodes: &[Node]) -> usize {
    if names.is_empty() { nodes.iter().map(|n| n.axis + 1).max().unwrap_or(2) } else { names.len() }
}

fn pct(px: f64) -> f64 {
    px / hive::SIZE * 100.0
}

/// A hive plot. Hover, touch or focus a node to light its links and the nodes at their
/// other ends. Focus the chart and use the arrow keys to step through the nodes axis by
/// axis, centre outwards; Home and End jump, Escape clears.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn HivePlotChart(
    /// The nodes: an id, the axis for its type, and a value that sets how far out it sits.
    nodes: ReadSignal<Vec<Node>>,
    /// The links, by node id. Links within one axis and to unknown ids are ignored.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Service calls by tier". Used as the accessible name.
    label: String,
    /// Axis names, by axis number. Two to four; their count sets the number of axes.
    #[props(default)]
    axes: Vec<String>,
    /// Name of the value that places nodes along an axis, for the tooltip and table.
    #[props(default = "Value".to_string())]
    value_label: String,
    /// Formats node values for the tooltip, axis ends and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let lay = use_memo(use_reactive((&axes,), move |(axes,)| {
        let nodes = nodes.read();
        hive::layout(&nodes, &links.read(), axis_count(&axes, &nodes))
    }));
    let lay = lay.read();
    let order = hive::order(&lay);
    let n = lay.nodes.len();
    let current = active().filter(|&i| i < n);
    let near = current.map(|i| hive::neighbours(&lay, i));
    let lit = |i: usize| near.as_ref().is_some_and(|x| x.get(i).copied().unwrap_or(false));
    let aname = |a: usize| axis_name(&axes, a);
    let linked = |i: usize| {
        let near = hive::neighbours(&lay, i);
        lay.nodes
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i && near[*j])
            .map(|(_, n)| n.id.clone())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let tip = current.map(|i| lay.nodes[i].clone());
    let announcement = tip.as_ref().map(|node| {
        format!("{}, {}: {value_label} {}, {} links", node.id, aname(node.axis), fmt(node.value), node.degree)
    });
    let (lo, hi) = lay.domain;
    let keys = order.clone();

    rsx! {
        figure {
            "data-scope": "hive-plot-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the nodes.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    let pos = active().and_then(|a| keys.iter().position(|&o| o == a));
                    if let Some(next) = nav::list_key(&e.key().to_string(), pos, keys.len()) {
                        e.prevent_default();
                        active.set(next.map(|p| keys[p]));
                    }
                },
                svg { view_box: "0 0 {hive::SIZE} {hive::SIZE}", "aria-hidden": "true",
                    g { "data-part": "axes",
                        for (k, a) in lay.axes.iter().enumerate() {
                            line {
                                key: "{k}",
                                "data-part": "axis",
                                "pathLength": "1",
                                x1: "{a.from.0}",
                                y1: "{a.from.1}",
                                x2: "{a.to.0}",
                                y2: "{a.to.1}",
                                style: "--i: {k};",
                            }
                            text {
                                "data-part": "axis-label",
                                x: "{a.label.0}",
                                y: "{a.label.1}",
                                text_anchor: a.anchor,
                                dominant_baseline: "central",
                                {aname(k)}
                            }
                        }
                    }
                    g { "data-part": "links",
                        for (k, e) in lay.edges.iter().enumerate() {
                            path {
                                key: "{k}",
                                "data-part": "link",
                                "pathLength": "1",
                                "data-highlighted": (current == Some(e.source) || current == Some(e.target)).then_some(""),
                                d: "{e.path}",
                                stroke: colour(lay.nodes[e.source].axis),
                                style: "--i: {k.min(40)};",
                            }
                        }
                    }
                    g { "data-part": "nodes",
                        for (i, node) in lay.nodes.iter().enumerate() {
                            g {
                                key: "{node.id}",
                                "data-part": "node",
                                "data-axis": "{node.axis}",
                                "data-highlighted": lit(i).then_some(""),
                                "data-selected": (current == Some(i)).then_some(""),
                                style: "--i: {i.min(40)};",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                circle { "data-part": "hit", cx: "{node.x}", cy: "{node.y}", r: "10" }
                                circle { "data-part": "dot", cx: "{node.x}", cy: "{node.y}", r: "4.5", fill: colour(node.axis) }
                            }
                        }
                    }
                }
                if let Some(node) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if pct(node.x) > 50.0 { "left" } else { "right" },
                        style: "left: {pct(node.x):.3}%; top: {pct(node.y):.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour(node.axis)};" }
                            "{node.id}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Type" }
                                span { "data-part": "tooltip-value", {aname(node.axis)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "{value_label}" }
                                span { "data-part": "tooltip-value", {fmt(node.value)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Links" }
                                span { "data-part": "tooltip-value", "{node.degree}" }
                            }
                        }
                    }
                }
            }
            p { "data-part": "scale-note", "aria-hidden": "true",
                "{value_label}: {fmt(lo)} at the centre to {fmt(hi)} at the tips"
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Node" }
                        th { scope: "col", "Type" }
                        th { scope: "col", "{value_label}" }
                        th { scope: "col", "Links" }
                        th { scope: "col", "Linked to" }
                    }
                }
                tbody {
                    for i in order.iter().copied() {
                        tr {
                            th { scope: "row", "{lay.nodes[i].id}" }
                            td { {aname(lay.nodes[i].axis)} }
                            td { {fmt(lay.nodes[i].value)} }
                            td { "{lay.nodes[i].degree}" }
                            td { {linked(i)} }
                        }
                    }
                }
            }
        }
    }
}
