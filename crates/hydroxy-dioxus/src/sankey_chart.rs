//! `SankeyChart`: flows between stages, link width by value. Layout comes from
//! `hydroxy_charts::sankey`. Hover, touch or focus a node or link to light its whole path.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, nav, sankey};

use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::sankey::Link;

/// A flow diagram. Nodes sit in columns by longest path and link widths follow their
/// values. Hover or touch a node or link to light everything upstream and downstream
/// and read its values; focus the chart and use the arrow keys to step through the nodes
/// and then the links, Home and End to jump, Escape to clear.
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
    /// Formats values for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    // Marks are the nodes first, then the links.
    let mut active = use_signal(|| None::<usize>);
    let lay = sankey::layout(&links(), box_w, box_h, 12.0, 14.0, 6);
    let n = lay.nodes.len();
    let marks = n + lay.links.len();
    let current = active().filter(|&m| m < marks);
    let lit =
        current.map(|m| if m < n { sankey::highlight_node(&lay, m) } else { sankey::highlight_link(&lay, m - n) });
    let node_lit = |i: usize| lit.as_ref().is_some_and(|x| x.nodes.get(i).copied().unwrap_or(false)).then_some("");
    let link_lit = |k: usize| lit.as_ref().is_some_and(|x| x.links.get(k).copied().unwrap_or(false)).then_some("");
    let last = lay.nodes.iter().map(|n| n.column).max().unwrap_or(0);
    let shown = current.and_then(|m| Some((sankey::tooltip(&lay, m, (box_w, box_h), fmt)?, sankey::swatch(&lay, m)?)));
    let announcement = shown.as_ref().map(|(tip, _)| tip.announce());

    rsx! {
        figure {
            "data-scope": "sankey-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the nodes and links.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), marks) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 {width} {height}", "aria-hidden": "true",
                    g { "data-part": "links",
                        for (k, r) in lay.links.iter().enumerate() {
                            path {
                                key: "{k}",
                                "data-part": "link",
                                "data-highlighted": link_lit(k),
                                style: "--i: {k};",
                                d: "{r.path}",
                                stroke: colour(r.source),
                                stroke_width: "{r.width}",
                                onpointerenter: move |_| active.set(Some(n + k)),
                                onpointerdown: move |_| active.set(Some(n + k)),
                            }
                        }
                    }
                    g { "data-part": "nodes",
                        for (i, node) in lay.nodes.iter().enumerate() {
                            g {
                                key: "{i}",
                                "data-part": "node",
                                "data-highlighted": node_lit(i),
                                "data-selected": (current == Some(i)).then_some(""),
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                rect {
                                    x: "{node.x}",
                                    y: "{node.y}",
                                    width: "{lay.node_width}",
                                    height: "{node.height.max(1.0)}",
                                    rx: "2",
                                    fill: colour(i),
                                }
                                text {
                                    "data-part": "node-label",
                                    x: if node.column == last && last > 0 { node.x - 6.0 } else { node.x + lay.node_width + 6.0 },
                                    y: "{node.y + node.height / 2.0}",
                                    text_anchor: if node.column == last && last > 0 { "end" } else { "start" },
                                    dominant_baseline: "middle",
                                    "{node.name}"
                                    tspan { "data-part": "node-value", dx: "6", {fmt(node.value)} }
                                }
                            }
                        }
                    }
                }
                if let Some((tip, swatch)) = shown {
                    ChartTip { tip, swatch, anchored: true }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
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
