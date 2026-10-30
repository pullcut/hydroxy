//! `EdgeBundlingChart`: hierarchical edge bundling. The leaves of a tree sit round a
//! circle and each link follows the tree between them, so links between the same
//! branches gather into bundles. Geometry comes from `hydroxy_charts::bundle`.

use dioxus::prelude::*;
use hydroxy_charts::bundle::{self, Role};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav;

use crate::callbacks::use_hover;

pub use hydroxy_charts::bundle::{Link, Tree};

/// Links out of and into leaf `i`, as names.
fn ends(b: &bundle::Bundle, i: usize) -> (Vec<String>, Vec<String>) {
    let out = b.edges.iter().filter(|e| e.source == i).map(|e| b.leaves[e.target].name.clone()).collect();
    let inn = b.edges.iter().filter(|e| e.target == i).map(|e| b.leaves[e.source].name.clone()).collect();
    (out, inn)
}

/// An edge-bundling diagram. Hover, touch or focus a leaf to see what it links to in one
/// colour and what links to it in another. Focus the chart and use the arrow keys to go
/// round the circle, Home and End to jump, Escape to clear.
#[component]
pub fn EdgeBundlingChart(
    /// The hierarchy. Its leaves go round the circle in order; its top-level branches are
    /// set apart by a gap. Leaf values are ignored.
    tree: ReadSignal<Tree>,
    /// Directed links between leaves, by name. Values are ignored.
    links: ReadSignal<Vec<Link>>,
    /// What the chart shows, such as "Module imports". Used as the accessible name.
    label: String,
    /// Bundling strength from 0 (straight lines) to 1 (links follow the tree exactly).
    #[props(default = 0.85)]
    beta: f64,
    /// Drawing width and height in px; the chart scales down to fit its container.
    #[props(default = 600)]
    size: u16,
    /// What a link means read forwards, for the legend and tooltip.
    #[props(default = "Links to".to_string())]
    out_label: String,
    /// What a link means read backwards, for the legend and tooltip.
    #[props(default = "Linked from".to_string())]
    in_label: String,
    /// Called when the pointer or keyboard moves to a leaf, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a leaf.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let mut active = use_signal(|| None::<usize>);
    let lay = use_memo(use_reactive((&beta, &size), move |(beta, size)| {
        bundle::layout(&tree.read(), &links.read(), f64::from(size), beta)
    }));
    let lay = lay.read();
    let n = lay.leaves.len();
    let current = active().filter(|&i| i < n);
    let datums: Vec<EventDatum> = if on_hover.is_some() || on_select.is_some() {
        let mut degree = vec![0.0_f64; n];
        for e in &lay.edges {
            degree[e.source] += 1.0;
            degree[e.target] += 1.0;
        }
        lay.leaves
            .iter()
            .zip(degree)
            .enumerate()
            .map(|(i, (l, value))| EventDatum {
                series: l.group,
                index: i,
                label: Some(l.name.clone()),
                value,
                node: None,
            })
            .collect()
    } else {
        Vec::new()
    };
    let now = current.and_then(|i| datums.get(i).cloned());
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let key_now = now;
    let roles = current.map(|i| bundle::roles(&lay, i));
    let role_of = |j: usize| roles.as_ref().and_then(|r| r.get(j).copied().flatten()).map(Role::attr);
    let edge_role = |s: usize, t: usize| match current {
        Some(i) if i == s => Some("out"),
        Some(i) if i == t => Some("in"),
        _ => None,
    };
    let tip = current.map(|i| (lay.leaves[i].clone(), lay.groups[lay.leaves[i].group].clone(), ends(&lay, i)));
    let announcement = tip.as_ref().map(|(leaf, group, (out, inn))| {
        format!(
            "{}, {group}: {} {}, {} {}",
            leaf.name,
            out_label.to_lowercase(),
            out.len(),
            in_label.to_lowercase(),
            inn.len()
        )
    });
    let side = f64::from(size);

    rsx! {
        figure {
            "data-scope": "edge-bundling-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {size}px;",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to go round the circle.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter
                        && let (Some(on_select), Some(d)) = (on_select, key_now.clone())
                    {
                        on_select.call(d);
                    }
                    if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { view_box: "0 0 {size} {size}", "aria-hidden": "true",
                    g { "data-part": "links",
                        for (k, e) in lay.edges.iter().enumerate() {
                            path {
                                key: "{k}",
                                "data-part": "link",
                                "pathLength": "1",
                                "data-role": edge_role(e.source, e.target),
                                d: "{e.path}",
                                style: "--i: {k}; --ox-n: {lay.edges.len()};",
                            }
                        }
                    }
                    g { "data-part": "leaves",
                        for (i, leaf) in lay.leaves.iter().enumerate() {
                            g {
                                key: "{i}",
                                "data-part": "leaf",
                                "data-group": "{leaf.group}",
                                "data-role": role_of(i),
                                "data-selected": (current == Some(i)).then_some(""),
                                onpointerenter: move |_| active.set(Some(i)),
                                onpointerdown: move |_| active.set(Some(i)),
                                onclick: {
                                    let picked = datums.get(i).cloned();
                                    move |_| {
                                        if let (Some(on_select), Some(d)) = (on_select, picked.clone()) {
                                            on_select.call(d);
                                        }
                                    }
                                },
                                circle { "data-part": "hit", cx: "{leaf.x}", cy: "{leaf.y}", r: "8" }
                                circle { "data-part": "dot", cx: "{leaf.x}", cy: "{leaf.y}", r: "2.5" }
                                text {
                                    "data-part": "leaf-label",
                                    transform: "{leaf.transform}",
                                    text_anchor: leaf.anchor,
                                    dominant_baseline: "central",
                                    "{leaf.name}"
                                }
                            }
                        }
                    }
                }
                if let Some((leaf, group, (out, inn))) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if leaf.x / side > 0.5 { "left" } else { "right" },
                        style: "left: {leaf.x / side * 100.0:.3}%; top: {leaf.y / side * 100.0:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label", "{leaf.name}" }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Group" }
                                span { "data-part": "tooltip-value", "{group}" }
                            }
                            li { "data-part": "tooltip-row", "data-role": "out",
                                span { "data-part": "tooltip-name",
                                    span { "data-part": "swatch" }
                                    "{out_label}"
                                }
                                span { "data-part": "tooltip-value", "{out.len()}" }
                            }
                            li { "data-part": "tooltip-row", "data-role": "in",
                                span { "data-part": "tooltip-name",
                                    span { "data-part": "swatch" }
                                    "{in_label}"
                                }
                                span { "data-part": "tooltip-value", "{inn.len()}" }
                            }
                        }
                    }
                }
            }
            ul { "data-part": "legend", "aria-hidden": "true",
                li { "data-part": "legend-item", "data-role": "out",
                    span { "data-part": "swatch" }
                    "{out_label}"
                }
                li { "data-part": "legend-item", "data-role": "in",
                    span { "data-part": "swatch" }
                    "{in_label}"
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Leaf" }
                        th { scope: "col", "Group" }
                        th { scope: "col", "{out_label}" }
                        th { scope: "col", "{in_label}" }
                    }
                }
                tbody {
                    for (i, leaf) in lay.leaves.iter().enumerate() {
                        tr {
                            th { scope: "row", "{leaf.name}" }
                            td { "{lay.groups[leaf.group]}" }
                            td { {ends(&lay, i).0.join(", ")} }
                            td { {ends(&lay, i).1.join(", ")} }
                        }
                    }
                }
            }
        }
    }
}
