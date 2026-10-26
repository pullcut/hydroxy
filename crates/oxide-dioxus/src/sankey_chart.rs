//! `SankeyChart`: flows between stages, link width by value. Layout comes from
//! `oxide_charts::sankey`. Hover or focus a node or link to light its whole path.

use dioxus::prelude::*;
use oxide_charts::{format_value, sankey};

pub use oxide_charts::sankey::Link;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

#[derive(Clone, Copy, PartialEq)]
enum Active {
    Node(usize),
    Link(usize),
}

/// A flow diagram. Nodes sit in columns by longest path and link widths follow their
/// values. Tab through the nodes to light up everything upstream and downstream.
/// Links that would close a cycle are ignored.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SankeyChart(
    /// The flows. Nodes are created from the names, in order of first appearance.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Signups to plan". Used as the accessible name.
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[props(default = 720)]
    width: u16,
    /// Drawing height in px.
    #[props(default = 340)]
    height: u16,
    /// Formats values for labels and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<Active>);
    let lay = sankey::layout(&links(), f64::from(width), f64::from(height), 12.0, 14.0, 6);
    let lit = active().map(|a| match a {
        Active::Node(i) => sankey::highlight_node(&lay, i),
        Active::Link(k) => sankey::highlight_link(&lay, k),
    });
    let node_lit = |i: usize| lit.as_ref().is_some_and(|x| x.nodes.get(i).copied().unwrap_or(false)).then_some("");
    let link_lit = |k: usize| lit.as_ref().is_some_and(|x| x.links.get(k).copied().unwrap_or(false)).then_some("");
    let last = lay.nodes.iter().map(|n| n.column).max().unwrap_or(0);

    rsx! {
        figure {
            "data-scope": "sankey-chart",
            "data-part": "root",
            "data-active": active().map(|_| ""),
            style: "max-width: {width}px;",
            svg { view_box: "0 0 {width} {height}", role: "group", "aria-label": "{label}",
                g { "data-part": "links",
                    for (k, r) in lay.links.iter().enumerate() {
                        path {
                            key: "{k}",
                            "data-part": "link",
                            "data-highlighted": link_lit(k),
                            d: "{r.path}",
                            stroke: colour(r.source),
                            stroke_width: "{r.width}",
                            onpointerenter: move |_| active.set(Some(Active::Link(k))),
                            onpointerleave: move |_| active.set(None),
                            title { {format!("{} to {}: {}", lay.nodes[r.source].name, lay.nodes[r.target].name, fmt(r.value))} }
                        }
                    }
                }
                g { "data-part": "nodes",
                    for (i, n) in lay.nodes.iter().enumerate() {
                        g {
                            key: "{i}",
                            "data-part": "node",
                            "data-highlighted": node_lit(i),
                            tabindex: "0",
                            role: "img",
                            "aria-label": format!("{}: {}", n.name, fmt(n.value)),
                            onpointerenter: move |_| active.set(Some(Active::Node(i))),
                            onpointerleave: move |_| active.set(None),
                            onfocus: move |_| active.set(Some(Active::Node(i))),
                            onblur: move |_| active.set(None),
                            rect {
                                x: "{n.x}",
                                y: "{n.y}",
                                width: "{lay.node_width}",
                                height: "{n.height.max(1.0)}",
                                rx: "2",
                                fill: colour(i),
                            }
                            text {
                                "data-part": "node-label",
                                x: if n.column == last && last > 0 { n.x - 6.0 } else { n.x + lay.node_width + 6.0 },
                                y: "{n.y + n.height / 2.0}",
                                text_anchor: if n.column == last && last > 0 { "end" } else { "start" },
                                dominant_baseline: "middle",
                                "{n.name}"
                                tspan { "data-part": "node-value", dx: "6", {fmt(n.value)} }
                            }
                        }
                    }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "From" }
                        th { scope: "col", "To" }
                        th { scope: "col", "Value" }
                    }
                }
                tbody {
                    for r in lay.links.iter() {
                        tr {
                            td { "{lay.nodes[r.source].name}" }
                            td { "{lay.nodes[r.target].name}" }
                            td { {fmt(r.value)} }
                        }
                    }
                }
            }
        }
    }
}
