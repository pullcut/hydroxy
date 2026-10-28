//! `TreemapChart`: a tree as nested rectangles sized by value, with click-to-zoom and
//! a breadcrumb. Layout comes from `hydroxy_charts::treemap`.

use dioxus::prelude::*;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, treemap};
use std::rc::Rc;

use crate::hierarchy_kit::{Breadcrumb, drill};
use crate::plot_kit::{ChartTip, colour, pct};

pub use hydroxy_charts::hierarchy::Tree;

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
            Breadcrumb { crumbs, go: Callback::new(go) }
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
                    let mut enter = go;
                    let up_path = up.clone();
                    let handled = drill(
                        &e.key().to_string(),
                        active,
                        &zoomable,
                        last == 0,
                        |i| enter(paths[i].clone()),
                        || {
                            if let Some(p) = up_path {
                                go(p);
                            }
                        },
                    );
                    if handled {
                        e.prevent_default();
                    }
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
                        style: "left: {pct(c.x / w)}; top: {pct(c.y / h)}; width: {pct(c.w / w)}; height: {pct(c.h / h)}; --c: {colour(c.colour)}; --i: {i.min(40)};",
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
                    ChartTip {
                        tip: Tip {
                            x: ax,
                            y: ay,
                            title: cell.name.clone(),
                            rows: vec![
                                TipRow::new(None, "Value", fmt(cell.value)),
                                TipRow::new(None, format!("Share of {level_name}"), format!("{:.0}%", share(cell.value) * 100.0)),
                            ],
                        },
                        swatch: colour(cell.colour),
                        anchored: true,
                        side: if ax > 0.5 { "left" } else { "right" },
                        style: format!("left: {:.3}%; top: {:.3}%;", ax * 100.0, ay * 100.0),
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
