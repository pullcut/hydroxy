//! `ArcDiagramChart`: nodes in a row, each link a semicircle above the line. Ordering
//! by group keeps a cluster's arcs short, so links between clusters stand out as the
//! wide ones. Geometry comes from `hydroxy_charts::arc_diagram`.

use dioxus::prelude::*;
use hydroxy_charts::arc_diagram::{self, colour};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{format_value, nav};

use crate::rank_kit::{GroupLegend, distinct, group_name};

pub use hydroxy_charts::arc_diagram::{Link, Node, Order};

/// An arc diagram. Hover, touch or focus a node to light its arcs and the nodes at their
/// other ends. Focus the chart and use the arrow keys to step through the nodes from left
/// to right, Home and End to jump, Escape to clear.
#[component]
pub fn ArcDiagramChart(
    /// The nodes. Ids must be unique; they are also the labels.
    nodes: ReadSignal<Vec<Node>>,
    /// The links, by node id. Links to unknown ids and self-links are ignored.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Characters who share a scene". Used as the accessible name.
    label: String,
    /// Names for the groups, by group number, for the legend, tooltip and table.
    /// Default: "Group 1", "Group 2" and so on.
    #[props(default)]
    groups: Vec<String>,
    /// How nodes are ordered along the line. Default: by group, most-linked first.
    #[props(default)]
    order: Order,
    /// Arc width follows link value. Turn off when values are all alike.
    #[props(default = true)]
    weighted: bool,
    /// Drawing width in px; the chart scales down to fit its container.
    #[props(default = 720)]
    width: u16,
    /// Formats link values for the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let lay = use_memo(use_reactive((&order, &weighted, &width), move |(order, weighted, width)| {
        arc_diagram::layout(&nodes.read(), &links.read(), f64::from(width), order, weighted)
    }));
    let lay = lay.read();
    let n = lay.nodes.len();
    let current = active().filter(|&i| i < n);
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|i| EventDatum {
            series: lay.nodes[i].group,
            ..EventDatum::category(i, lay.nodes[i].id.clone(), lay.nodes[i].weight)
        })
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let near = current.map(|i| arc_diagram::neighbours(&lay, i));
    let lit = |i: usize| near.as_ref().is_some_and(|x| x.get(i).copied().unwrap_or(false));
    let group_ids = distinct(lay.nodes.iter().map(|n| n.group));
    let gname = |g: usize| group_name(&groups, g);
    let order_v = lay.order.clone();
    let base = lay.baseline;
    let tip = current.map(|i| (lay.nodes[i].clone(), arc_diagram::linked(&lay, i)));
    let announcement = tip.as_ref().map(|(node, _)| {
        format!("{}, {}: {} links, weight {}", node.id, gname(node.group), node.degree, fmt(node.weight))
    });

    rsx! {
        figure {
            "data-scope": "arc-diagram-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{nav::hinted(&label, nav::HINT_NODES)}",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    let pos = active().and_then(|a| order_v.iter().position(|&o| o == a));
                    if let Some(next) = nav::list_key(&e.key().to_string(), pos, order_v.len()) {
                        e.prevent_default();
                        active.set(next.map(|p| order_v[p]));
                    }
                },
                svg { view_box: "0 0 {lay.width} {lay.height:.0}", "aria-hidden": "true",
                    line {
                        "data-part": "baseline",
                        x1: "{arc_diagram::PAD - 12.0}",
                        x2: "{lay.width - arc_diagram::PAD + 12.0}",
                        y1: "{base}",
                        y2: "{base}",
                    }
                    g { "data-part": "arcs",
                        for (k, a) in lay.arcs.iter().enumerate() {
                            path {
                                key: "{a.source}-{a.target}-{k}",
                                "data-part": "arc",
                                "pathLength": "1",
                                "data-highlighted": (current == Some(a.source) || current == Some(a.target)).then_some(""),
                                d: "{a.path}",
                                stroke: colour(lay.nodes[a.source].group),
                                stroke_width: "{a.width:.2}",
                                style: "--i: {k.min(40)};",
                            }
                        }
                    }
                    g { "data-part": "nodes",
                        for (slot, i) in lay.order.iter().copied().enumerate() {
                            g {
                                key: "{lay.nodes[i].id}",
                                "data-part": "node",
                                "data-group": "{lay.nodes[i].group}",
                                "data-highlighted": lit(i).then_some(""),
                                "data-selected": (current == Some(i)).then_some(""),
                                style: "--i: {slot.min(40)};",
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                onclick: move |_| {
                                    active.set(Some(i));
                                    crate::callbacks::select_active(Some(datum), on_select);
                                },
                                circle { "data-part": "hit", cx: "{lay.nodes[i].x}", cy: "{base}", r: "12" }
                                circle { "data-part": "dot", cx: "{lay.nodes[i].x}", cy: "{base}", r: "{lay.nodes[i].radius}", fill: colour(lay.nodes[i].group) }
                                text {
                                    "data-part": "node-label",
                                    transform: "translate({lay.nodes[i].x - 3.0:.2} {base + lay.nodes[i].radius + 8.0:.2}) rotate(45)",
                                    "{lay.nodes[i].id}"
                                }
                            }
                        }
                    }
                }
                if let Some((node, others)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if node.x / lay.width > 0.5 { "left" } else { "right" },
                        style: "left: {node.x / lay.width * 100.0:.3}%; top: {base / lay.height * 100.0:.3}%;",
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
                                span { "data-part": "tooltip-value", {fmt(node.weight)} }
                            }
                        }
                        if !others.is_empty() {
                            p { "data-part": "tooltip-note",
                                {others.iter().take(4).cloned().collect::<Vec<_>>().join(", ")}
                                if others.len() > 4 {
                                    " and {others.len() - 4} more"
                                }
                            }
                        }
                    }
                }
            }
            GroupLegend {
                groups: group_ids,
                name: Callback::new({
                    let names = groups.clone();
                    move |g| group_name(&names, g)
                }),
                colour: Callback::new(colour),
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
                    for i in lay.order.iter().copied() {
                        tr {
                            th { scope: "row", "{lay.nodes[i].id}" }
                            td { {gname(lay.nodes[i].group)} }
                            td { "{lay.nodes[i].degree}" }
                            td { {fmt(lay.nodes[i].weight)} }
                            td { {arc_diagram::linked(&lay, i).join(", ")} }
                        }
                    }
                }
            }
        }
    }
}
