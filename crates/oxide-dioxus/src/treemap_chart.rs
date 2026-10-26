//! `TreemapChart`: a tree as nested rectangles sized by value, with click-to-zoom and
//! a breadcrumb. Layout comes from `oxide_charts::treemap`.

use dioxus::prelude::*;
use oxide_charts::{format_value, treemap};
use std::rc::Rc;

pub use oxide_charts::hierarchy::Tree;

fn pct(v: f64, of: f64) -> String {
    format!("{:.3}%", v / of * 100.0)
}

/// A treemap. Cells are sized by value and the children of big cells are drawn inside
/// them. Click, or press Enter on, a cell with children to zoom into it; use the
/// breadcrumb to climb back out.
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
    /// Formats values for cells and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (w, h) = (800.0, f64::from(height));
    let mut zoom = use_signal(Vec::<usize>::new);
    let mut plot = use_signal(|| None::<Rc<MountedData>>);
    let tree = root();
    let cells = treemap::layout(&tree, &zoom(), w, h);
    let crumbs = tree.crumbs(&zoom());
    let last = crumbs.len() - 1;
    let mut go = move |path: Vec<usize>| {
        zoom.set(path);
        // The cells are rebuilt, so park focus on the plot rather than losing it.
        if let Some(m) = plot() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };

    rsx! {
        figure { "data-scope": "treemap-chart", "data-part": "root", "aria-label": "{label}",
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
                tabindex: "-1",
                style: "aspect-ratio: 800 / {height};",
                onmounted: move |e| plot.set(Some(e.data())),
                for c in cells {
                    button {
                        key: "{c.path:?}",
                        r#type: "button",
                        "data-part": "cell",
                        "data-depth": "{c.depth}",
                        "data-nested": c.nested.then_some(""),
                        "data-leaf": (!c.zoomable).then_some(""),
                        "aria-disabled": (!c.zoomable).then_some("true"),
                        "aria-label": format!("{}: {}{}", c.name, fmt(c.value), if c.zoomable { ", has parts, press Enter to zoom in" } else { "" }),
                        style: "left: {pct(c.x, w)}; top: {pct(c.y, h)}; width: {pct(c.w, w)}; height: {pct(c.h, h)}; --c: var(--chart-{c.colour % 6 + 1});",
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
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Item" }
                        th { scope: "col", "Value" }
                    }
                }
                tbody {
                    for (name, v) in tree.flatten() {
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
