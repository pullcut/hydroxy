//! `IcicleChart`: a tree as rows of rectangles, one row per level, each cell as wide
//! as its share of the zoomed node. Click a cell to drill into it. Layout comes from
//! `oxide_charts::partition`, which the flame graph shares.

use dioxus::prelude::*;
use oxide_charts::{format_value, nav, partition};
use std::rc::Rc;

pub use oxide_charts::hierarchy::Tree;

const WIDTH: f64 = 800.0;

fn colour(slot: Option<usize>) -> String {
    slot.map_or_else(|| "var(--chart-muted)".to_string(), |s| format!("var(--chart-{})", s + 1))
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// An icicle chart. Hover, touch or focus the chart and use the arrow keys to read each
/// cell; click a cell with children, or press Enter on it, to drill in. Escape, the top
/// row or the breadcrumb climb back out.
#[component]
pub fn IcicleChart(
    /// The tree. Branch values come from their children.
    root: ReadSignal<Tree>,
    /// What the chart shows, such as "Bundle size by module". Used as the accessible name.
    label: String,
    /// Plot height in px at an 800px drawing width.
    #[props(default = 320)]
    height: u16,
    /// Formats values for the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    rsx! {
        PartitionChart { root, label, height, format, highlight: None }
    }
}

/// The icicle and flame graph body. `highlight` is `Some` for a flame graph: rows run
/// bottom-up, branches keep self time, cells colour by name and matches stand out.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::many_single_char_names, reason = "x, y, w for geometry read best short")]
#[component]
pub(crate) fn PartitionChart(
    root: ReadSignal<Tree>,
    label: String,
    height: u16,
    format: Option<Callback<f64, String>>,
    highlight: Option<ReadSignal<String>>,
) -> Element {
    let flame = highlight.is_some();
    let query = highlight.map(|h| h()).unwrap_or_default();
    let query = query.trim().to_string();
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let plot_h = f64::from(height);
    let mut zoom = use_signal(Vec::<usize>::new);
    let mut active = use_signal(|| None::<usize>);
    let mut marks = use_signal(|| None::<Rc<MountedData>>);
    let tree = root();
    let l = partition::layout(&tree, &zoom(), flame);
    let crumbs = tree.crumbs(&zoom());
    let last = crumbs.len() - 1;
    let up = (last > 0).then(|| crumbs[last - 1].1.clone());
    let current = active().filter(|&i| i < l.cells.len());
    let mut go = move |path: Vec<usize>| {
        active.set(None);
        zoom.set(path);
        if let Some(m) = marks() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
    let fill = move |c: &partition::Cell| if flame { colour(Some(partition::name_slot(&c.name))) } else { colour(c.colour) };
    let whole = l.cells.first().map_or(0.0, |c| c.value);
    let share = move |v: f64| if whole > 0.0 { v / whole * 100.0 } else { 0.0 };
    let targets: Vec<Option<Vec<usize>>> = l.cells.iter().map(|c| if c.depth == 0 { up.clone() } else { c.zoomable.then(|| c.path.clone()) }).collect();
    let matches = l.cells.iter().filter(|c| partition::matches(&c.name, &query)).count();
    let announcement = current.map_or_else(
        || {
            if query.is_empty() {
                String::new()
            } else {
                format!("{matches} {} match “{query}”", if matches == 1 { "frame" } else { "frames" })
            }
        },
        |i| {
            let c = &l.cells[i];
            let own = if flame { format!(", self {}", fmt(c.self_value)) } else { String::new() };
            let hint = match (c.depth, c.zoomable) {
                (0, _) if up.is_some() => ", press Enter to zoom out",
                (0, _) => "",
                (_, true) => ", press Enter to zoom in",
                _ => "",
            };
            format!("{}: {}{own}, {:.0}% of {}{hint}", c.name, fmt(c.value), share(c.value), l.cells[0].name)
        },
    );
    let legend = if flame { Vec::new() } else { tree.node_at(&tree.valid_path(&zoom())).map(partition::legend).unwrap_or_default() };
    let scope = if flame { "flame-graph-chart" } else { "icicle-chart" };
    let noun = if flame { "frames" } else { "cells" };
    let count = l.cells.len();
    let key_targets = targets.clone();
    let key_up = up.clone();
    let tip = current.map(|i| {
        let c = l.cells[i].clone();
        let (x, y, w, rh) = c.rect(l.levels, WIDTH, plot_h, flame);
        let pos = ((x + w / 2.0) / WIDTH, if flame { y / plot_h } else { (y + rh).min(plot_h) / plot_h });
        (c, pos)
    });

    rsx! {
        figure { "data-scope": scope, "data-part": "root", "data-searching": (!query.is_empty()).then_some(""),
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
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {WIDTH} {height}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to move between {noun}, Enter to zoom in, Escape to zoom out.",
                    onmounted: move |e| marks.set(Some(e.data())),
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        let key = e.key().to_string();
                        if key == "Enter" || key == " " {
                            if let Some(t) = current.and_then(|i| key_targets[i].clone()) {
                                e.prevent_default();
                                go(t);
                            }
                            return;
                        }
                        if key == "Escape" && current.is_none() {
                            if let Some(p) = key_up.clone() {
                                e.prevent_default();
                                go(p);
                            }
                            return;
                        }
                        let key = match key.as_str() {
                            "ArrowDown" => "ArrowRight",
                            "ArrowUp" => "ArrowLeft",
                            k => k,
                        };
                        if let Some(next) = nav::list_key(key, current, count) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for (i, c) in l.cells.iter().enumerate() {
                        {
                            let (x, y, w, rh) = c.rect(l.levels, WIDTH, plot_h, flame);
                            let text = (rh >= 16.0).then(|| partition::fit_label(&c.name, w, 6.5)).flatten();
                            let t = targets[i].clone();
                            rsx! {
                                g {
                                    key: "{c.path:?}",
                                    "data-part": "cell",
                                    "data-depth": "{c.depth}",
                                    "data-leaf": (!c.zoomable).then_some(""),
                                    "data-match": partition::matches(&c.name, &query).then_some(""),
                                    "data-active": (current == Some(i)).then_some(""),
                                    style: "--i: {c.depth.min(20)};",
                                    onpointerenter: move |_| active.set(Some(i)),
                                    onpointerdown: move |_| active.set(Some(i)),
                                    onclick: move |_| {
                                        if let Some(p) = t.clone() {
                                            go(p);
                                        }
                                    },
                                    rect { x: "{x:.2}", y: "{y + 0.5:.2}", width: "{w:.2}", height: "{(rh - 1.0).max(0.5):.2}", rx: "2", fill: fill(c) }
                                    if let Some(t) = text {
                                        text { "data-part": "label", x: "{x + 4.0:.2}", y: "{y + rh / 2.0:.2}", dominant_baseline: "central", "{t}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some((c, (fx, fy))) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if fx > 0.5 { "left" } else { "right" },
                        "data-vertical": if fy < 0.25 { "below" } else { "above" },
                        style: "left: {pct(fx)}; top: {pct(fy)};",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {fill(&c)};" }
                            "{c.name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", if flame { "Total" } else { "Value" } }
                                span { "data-part": "tooltip-value", {fmt(c.value)} }
                            }
                            if flame {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "Self" }
                                    span { "data-part": "tooltip-value", {fmt(c.self_value)} }
                                }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Share" }
                                span { "data-part": "tooltip-value", "{share(c.value):.1}%" }
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
                        th { scope: "col", if flame { "Frame" } else { "Node" } }
                        th { scope: "col", if flame { "Total" } else { "Value" } }
                        if flame {
                            th { scope: "col", "Self" }
                        }
                    }
                }
                tbody {
                    for (name, v, own) in partition::rows(&tree, flame) {
                        tr {
                            th { scope: "row", "{name}" }
                            td { {fmt(v)} }
                            if flame {
                                td { {fmt(own)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
