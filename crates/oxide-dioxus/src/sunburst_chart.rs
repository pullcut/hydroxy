//! `SunburstChart`: a tree as concentric rings, each arc's sweep its share of the
//! whole. Click an arc to zoom into it, the centre to climb out. Layout comes from
//! `oxide_charts::sunburst`.

use dioxus::prelude::*;
use oxide_charts::hierarchy::{Drill, drill_key};
use oxide_charts::{format_value, sunburst};
use std::rc::Rc;

pub use oxide_charts::hierarchy::Tree;

/// A sunburst. Hover or touch an arc to read it; click one with parts to zoom in, and
/// the centre to zoom back out. Focus the chart and use the arrow keys to step through
/// arcs, Enter to zoom in, Backspace to zoom out, Escape to clear. The centre shows the
/// active arc, or the current level.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SunburstChart(
    /// The tree. Branch values come from their children.
    root: ReadSignal<Tree>,
    /// What the chart shows, such as "Spend by team". Used as the accessible name.
    label: String,
    /// Diameter in px; the chart scales down to fit its container.
    #[props(default = 420)]
    size: u16,
    /// How many rings to draw, 1 to 4.
    #[props(default = 3)]
    depth: u8,
    /// Formats values for the centre, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let s = f64::from(size);
    let rings = usize::from(depth.clamp(1, 4));
    // The centre disc matches the layout's hole: 26% of the outer radius.
    let hole_pct = 2.0 * (s / 2.0 - 2.0) * 0.26 / s * 100.0;
    let mut zoom = use_signal(Vec::<usize>::new);
    let mut active = use_signal(|| None::<usize>);
    let mut notice = use_signal(String::new);
    let mut plot = use_signal(|| None::<Rc<MountedData>>);
    let tree = root();
    let lay = sunburst::layout(&tree, &zoom(), s, rings);
    let crumbs = tree.crumbs(&zoom());
    let last = crumbs.len() - 1;
    let current = active().filter(|&i| i < lay.arcs.len());
    let mut go = move |path: Vec<usize>| {
        let from = zoom();
        let t = root();
        let next = sunburst::layout(&t, &path, s, rings);
        // Climbing out picks the node we left; zooming in starts fresh.
        active.set(next.arcs.iter().position(|a| a.path == from));
        let name = t.crumbs(&path).pop().map(|c| c.0).unwrap_or_default();
        notice.set(format!("Showing {name}, {} parts", next.arcs.iter().filter(|a| a.depth == 1).count()));
        zoom.set(path);
        if let Some(m) = plot() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
    let up = (last > 0).then(|| crumbs[last - 1].1.clone());
    let up_key = up.clone();
    let (name, value) = match current.and_then(|i| lay.arcs.get(i)) {
        Some(a) => (a.name.clone(), a.value),
        None => (crumbs[last].0.clone(), tree.node_at(&tree.valid_path(&zoom())).map_or(0.0, Tree::total)),
    };
    let top = last == 0;
    let announcement = if let Some(a) = current.and_then(|i| lay.arcs.get(i)) {
        let hint = if a.zoomable { ", press Enter to zoom in" } else { "" };
        format!("{}: {}, {:.0}%{hint}", a.name, fmt(a.value), a.share * 100.0)
    } else {
        notice()
    };
    let zoomable: Vec<bool> = lay.arcs.iter().map(|a| a.zoomable).collect();
    let paths: Vec<Vec<usize>> = lay.arcs.iter().map(|a| a.path.clone()).collect();
    let tip = current.and_then(|i| lay.anchor(i).map(|xy| (lay.arcs[i].clone(), xy)));
    let level_name = crumbs[last].0.clone();
    let top_arcs: Vec<(usize, String, usize)> = lay.arcs.iter().enumerate().filter(|(_, a)| a.depth == 1).map(|(i, a)| (i, a.name.clone(), a.colour)).collect();
    let total = tree.total();

    rsx! {
        figure {
            "data-scope": "sunburst-chart",
            "data-part": "root",
            "aria-label": "{label}",
            "data-active": current.map(|_| ""),
            style: "max-width: {size}px;",
            nav { "data-part": "breadcrumb", "aria-label": "Zoom level",
                ol {
                    for (i, (name, path)) in crumbs.iter().cloned().enumerate() {
                        li { key: "{i}",
                            if i == last {
                                span { "data-part": "crumb", "aria-current": "location", "{name}" }
                            } else {
                                button { r#type: "button", "data-part": "crumb", onclick: move |_| go(path.clone()), "{name}" }
                            }
                        }
                    }
                }
            }
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through arcs, Enter to zoom in, Backspace to zoom out.",
                onmounted: move |e| plot.set(Some(e.data())),
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    match drill_key(&e.key().to_string(), active(), &zoomable, top) {
                        Some(Drill::Move(next)) => active.set(next),
                        Some(Drill::Into(i)) => go(paths[i].clone()),
                        Some(Drill::Up) => {
                            if let Some(p) = up_key.clone() {
                                go(p);
                            }
                        }
                        None => return,
                    }
                    e.prevent_default();
                },
                svg { view_box: "0 0 {size} {size}", "aria-hidden": "true",
                    for (i, a) in lay.arcs.iter().enumerate() {
                        path {
                            key: "{a.path:?}",
                            "data-part": "arc",
                            "data-depth": "{a.depth}",
                            "data-leaf": (!a.zoomable).then_some(""),
                            "data-active": (current == Some(i)).then_some(""),
                            d: "{a.path_d}",
                            fill: "var(--chart-{a.colour % 6 + 1})",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: {
                                let (path, zoomable) = (a.path.clone(), a.zoomable);
                                move |_| {
                                    if zoomable {
                                        go(path.clone());
                                    }
                                }
                            },
                        }
                    }
                }
                button {
                    r#type: "button",
                    "data-part": "center",
                    tabindex: "-1",
                    disabled: top,
                    "aria-label": if top { "Top level".to_string() } else { format!("Zoom out from {}", crumbs[last].0) },
                    style: "width: {hole_pct:.3}%;",
                    onclick: move |_| {
                        if let Some(p) = up.clone() {
                            go(p);
                        }
                    },
                    span { "data-part": "center-name", "{name}" }
                    span { "data-part": "center-value", {fmt(value)} }
                }
                if let Some((arc, (ax, ay))) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if ax > 0.5 { "right" } else { "left" },
                        style: "left: {ax * 100.0:.3}%; top: {ay * 100.0:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: var(--chart-{arc.colour % 6 + 1});" }
                            "{arc.name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {fmt(arc.value)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Share of {level_name}" }
                                span { "data-part": "tooltip-value", {format!("{:.0}%", arc.share * 100.0)} }
                            }
                        }
                    }
                }
            }
            if top_arcs.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i, name, colour) in top_arcs {
                        li {
                            key: "{i}",
                            "data-part": "legend-item",
                            "data-active": (current == Some(i)).then_some(""),
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerleave: move |_| active.set(None),
                            span { "data-part": "swatch", style: "background: var(--chart-{colour % 6 + 1});" }
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
                        th { scope: "col", "Item" }
                        th { scope: "col", "Value" }
                        th { scope: "col", "Share" }
                    }
                }
                tbody {
                    for (name, v) in tree.flatten() {
                        tr {
                            th { scope: "row", "{name}" }
                            td { {fmt(v)} }
                            td { {format!("{:.1}%", if total > 0.0 { v / total * 100.0 } else { 0.0 })} }
                        }
                    }
                }
            }
        }
    }
}
