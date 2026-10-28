//! `TreemapChart`: a tree as nested rectangles sized by value, with click-to-zoom and
//! a breadcrumb. Layout comes from `hydroxy_charts::treemap`.

use dioxus::prelude::*;
use hydroxy_charts::hierarchy::{Drill, drill_key};
use hydroxy_charts::{format_value, treemap};
use std::rc::Rc;

pub use hydroxy_charts::hierarchy::Tree;

fn pct(v: f64, of: f64) -> String {
    format!("{:.3}%", v / of * 100.0)
}

/// A treemap. Cells are sized by value and the children of big cells are drawn inside
/// them. Hover or touch a cell to read it; click one with parts to zoom into it and use
/// the breadcrumb to climb back out. Focus the plot and use the arrow keys to step
/// through cells, Enter to zoom in, Backspace to zoom out, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TreemapChart(
    /// The tree. Branch values come from their children.
    root: ReadSignal<Tree>,
    /// What the chart shows, such as "Spend by team". Used as the accessible name.
    label: String,
    /// Plot height in px at the drawing's native width of 800.
    #[props(default = 380)]
    height: u16,
    /// Formats values for cells, the tooltip and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (w, h) = (800.0, f64::from(height));
    let mut zoom = use_signal(Vec::<usize>::new);
    let mut active = use_signal(|| None::<usize>);
    let mut notice = use_signal(String::new);
    let mut plot = use_signal(|| None::<Rc<MountedData>>);
    let tree = root();
    let cells = treemap::layout(&tree, &zoom(), w, h);
    let crumbs = tree.crumbs(&zoom());
    let last = crumbs.len() - 1;
    let level_total = tree.node_at(&tree.valid_path(&zoom())).map_or(0.0, Tree::total);
    let share = move |v: f64| if level_total > 0.0 { v / level_total } else { 0.0 };
    let current = active().filter(|&i| i < cells.len());
    let mut go = move |path: Vec<usize>| {
        let from = zoom();
        let t = root();
        let next = treemap::layout(&t, &path, w, h);
        // Climbing out picks the node we left; zooming in starts fresh.
        active.set(next.iter().position(|x| x.path == from));
        let name = t.crumbs(&path).pop().map(|c| c.0).unwrap_or_default();
        notice.set(format!("Showing {name}, {} parts", next.iter().filter(|x| x.depth == 1).count()));
        zoom.set(path);
        // The cells are rebuilt, so park focus on the plot rather than losing it.
        if let Some(m) = plot() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
    let announcement = if let Some(i) = current {
        let x = &cells[i];
        let hint = if x.zoomable { ", press Enter to zoom in" } else { "" };
        format!("{}: {}, {:.0}%{hint}", x.name, fmt(x.value), share(x.value) * 100.0)
    } else {
        notice()
    };
    let zoomable: Vec<bool> = cells.iter().map(|x| x.zoomable).collect();
    let paths: Vec<Vec<usize>> = cells.iter().map(|x| x.path.clone()).collect();
    let up = (last > 0).then(|| crumbs[last - 1].1.clone());
    let level_name = crumbs[last].0.clone();
    let tip = current.map(|i| {
        let c = &cells[i];
        (c.clone(), (c.x + c.w / 2.0) / w, (c.y + c.h / 2.0) / h)
    });
    let total = tree.total();

    rsx! {
        figure { "data-scope": "treemap-chart", "data-part": "root", "aria-label": "{label}", "data-active": current.map(|_| ""),
            nav { "data-part": "breadcrumb", "aria-label": "Zoom level",
                ol {
                    for (i, (name, path)) in crumbs.into_iter().enumerate() {
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
                "aria-label": "{label}. Use the arrow keys to step through cells, Enter to zoom in, Backspace to zoom out.",
                style: "aspect-ratio: 800 / {height};",
                onmounted: move |e| plot.set(Some(e.data())),
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    match drill_key(&e.key().to_string(), active(), &zoomable, last == 0) {
                        Some(Drill::Move(next)) => active.set(next),
                        Some(Drill::Into(i)) => go(paths[i].clone()),
                        Some(Drill::Up) => {
                            if let Some(p) = up.clone() {
                                go(p);
                            }
                        }
                        None => return,
                    }
                    e.prevent_default();
                },
                for (i, c) in cells.into_iter().enumerate() {
                    div {
                        key: "{c.path:?}",
                        "data-part": "cell",
                        "data-depth": "{c.depth}",
                        "data-nested": c.nested.then_some(""),
                        "data-leaf": (!c.zoomable).then_some(""),
                        "data-active": (current == Some(i)).then_some(""),
                        "aria-hidden": "true",
                        style: "left: {pct(c.x, w)}; top: {pct(c.y, h)}; width: {pct(c.w, w)}; height: {pct(c.h, h)}; --c: var(--chart-{c.colour % 6 + 1}); --i: {i.min(40)};",
                        onpointerenter: move |_| active.set(Some(i)),
                        onpointerdown: move |_| active.set(Some(i)),
                        onclick: {
                            let (path, zoomable) = (c.path.clone(), c.zoomable);
                            move |_| {
                                if zoomable {
                                    go(path.clone());
                                }
                            }
                        },
                        if c.label {
                            span { "data-part": "cell-name", "{c.name}" }
                            span { "data-part": "cell-value", {fmt(c.value)} }
                        }
                    }
                }
                if let Some((cell, ax, ay)) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-anchored": "",
                        "data-side": if ax > 0.5 { "left" } else { "right" },
                        style: "left: {ax * 100.0:.3}%; top: {ay * 100.0:.3}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: var(--chart-{cell.colour % 6 + 1});" }
                            "{cell.name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {fmt(cell.value)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Share of {level_name}" }
                                span { "data-part": "tooltip-value", {format!("{:.0}%", share(cell.value) * 100.0)} }
                            }
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
