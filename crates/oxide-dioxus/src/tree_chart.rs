//! `TreeChart`: a tidy node-link tree, top-down or left-to-right, with collapsible
//! branches. Layout comes from `oxide_charts::tidy`.

use dioxus::prelude::*;
use oxide_charts::tidy::{self, TreeKey};
use oxide_charts::{format_value, partition};

pub use oxide_charts::Orientation;
pub use oxide_charts::hierarchy::Tree;

fn colour(slot: Option<usize>) -> String {
    slot.map_or_else(|| "var(--chart-muted)".to_string(), |s| format!("var(--chart-{})", s + 1))
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

const WIDTH: f64 = 800.0;

/// A tidy tree. Click a node with children to collapse or expand it. Focus the chart
/// and use the arrow keys as in a tree view: Up and Down step through visible nodes,
/// Right expands or enters a branch, Left collapses or climbs to the parent, Home and
/// End jump, Enter or Space toggles, Escape clears.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TreeChart(
    /// The tree. Branch values come from their children.
    root: ReadSignal<Tree>,
    /// What the chart shows, such as "Reporting lines". Used as the accessible name.
    label: String,
    /// Vertical (default) puts the root at the top; Horizontal puts it on the left.
    #[props(default)]
    orientation: Orientation,
    /// Plot height in px at an 800px drawing width.
    #[props(default = 360)]
    height: u16,
    /// Formats values for the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let plot_h = f64::from(height);
    let mut collapsed = use_signal(Vec::<Vec<usize>>::new);
    let mut active = use_signal(|| None::<usize>);
    let tree = root();
    let nodes = tidy::layout(&tree, &collapsed());
    let current = active().filter(|&i| i < nodes.len());
    let mut toggle = move |path: Vec<usize>| {
        collapsed.with_mut(|c| {
            if let Some(k) = c.iter().position(|p| *p == path) {
                c.remove(k);
            } else {
                c.push(path);
            }
        });
    };
    let child_count = |path: &[usize]| tree.node_at(path).map_or(0, |n| n.children.len());
    let announcement = current
        .map(|i| {
            let n = &nodes[i];
            let state = match (n.has_children, n.collapsed) {
                (false, _) => String::new(),
                (true, true) => format!(", collapsed, {} children hidden", child_count(&n.path)),
                (true, false) => format!(", expanded, {} children", child_count(&n.path)),
            };
            format!("{}: {}, level {}{state}", n.name, fmt(n.value), n.depth + 1)
        })
        .unwrap_or_default();
    let legend = partition::legend(&tree);
    let points: Vec<(f64, f64)> = nodes.iter().map(|n| n.point(orientation, WIDTH, plot_h)).collect();
    let links: Vec<(usize, String)> = nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| Some((n.depth, tidy::link(points[n.parent?], points[i], orientation))))
        .collect();
    let tip = current.map(|i| {
        let n = nodes[i].clone();
        let kids = child_count(&n.path);
        (n, points[i], kids)
    });
    let key_nodes = nodes.clone();

    rsx! {
        figure { "data-scope": "tree-chart", "data-part": "root", "data-orientation": orientation.attr(),
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {WIDTH} {height}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to move through the tree, Enter to collapse or expand a branch.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        let Some(action) = tidy::key(&e.key().to_string(), current, &key_nodes) else { return };
                        e.prevent_default();
                        match action {
                            TreeKey::Focus(next) => active.set(next),
                            TreeKey::Toggle(i) => toggle(key_nodes[i].path.clone()),
                        }
                    },
                    for (depth, d) in links {
                        path { "data-part": "link", d: "{d}", style: "--i: {depth.min(20)};" }
                    }
                    for (i, n) in nodes.iter().enumerate() {
                        {
                            let (x, y) = points[i];
                            let (lx, ly, anchor) = match orientation {
                                Orientation::Vertical => (x, y + 18.0, "middle"),
                                Orientation::Horizontal => (x + 10.0, y, "start"),
                            };
                            let path = n.has_children.then(|| n.path.clone());
                            rsx! {
                                g {
                                    key: "{n.path:?}",
                                    "data-part": "node",
                                    "data-collapsed": n.collapsed.then_some(""),
                                    "data-leaf": (!n.has_children).then_some(""),
                                    "data-active": (current == Some(i)).then_some(""),
                                    style: "--i: {n.depth.min(20)};",
                                    onpointerenter: move |_| active.set(Some(i)),
                                    onpointerdown: move |_| active.set(Some(i)),
                                    onclick: move |_| {
                                        if let Some(p) = path.clone() {
                                            toggle(p);
                                        }
                                    },
                                    circle { cx: "{x:.2}", cy: "{y:.2}", r: if n.collapsed { "6.5" } else { "5" }, style: "--c: {colour(n.colour)};" }
                                    text { "data-part": "label", x: "{lx:.2}", y: "{ly:.2}", text_anchor: anchor, dominant_baseline: "central", "{n.name}" }
                                }
                            }
                        }
                    }
                }
                if let Some((n, (x, y), kids)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if x / WIDTH > 0.5 { "left" } else { "right" },
                        "data-vertical": if y / plot_h < 0.25 { "below" } else { "above" },
                        style: "left: {pct(x / WIDTH)}; top: {pct(y / plot_h)};",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour(n.colour)};" }
                            "{n.name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {fmt(n.value)} }
                            }
                            if n.has_children {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "Children" }
                                    span { "data-part": "tooltip-value", {format!("{kids} {}", if n.collapsed { "hidden" } else { "shown" })} }
                                }
                            }
                        }
                    }
                }
            }
            if legend.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i, name) in legend.iter().enumerate() {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {colour(Some(i))};" }
                            "{name}"
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
                        th { scope: "col", "Value" }
                    }
                }
                tbody {
                    for (name, v, _) in partition::rows(&tree, false) {
                        tr {
                            th { scope: "row", "{name}" }
                            td { {fmt(v)} }
                        }
                    }
                }
            }
        }
    }
}
