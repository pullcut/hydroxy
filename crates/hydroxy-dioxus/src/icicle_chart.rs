//! `IcicleChart`: a tree as rows of rectangles, one row per level, each cell as wide
//! as its share of the zoomed node. Click a cell to drill into it. Layout comes from
//! `hydroxy_charts::partition`, which the flame graph shares.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow, colour_or_muted};
use hydroxy_charts::{format_value, partition};
use std::rc::Rc;

use crate::callbacks::use_hover;
use crate::hierarchy_kit::{Breadcrumb, PartitionLegend, ZoomKey, zoom_key};
use crate::plot_kit::{ChartTip, colour, pct};

pub use hydroxy_charts::hierarchy::Tree;

const WIDTH: f64 = 800.0;

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
    /// Called when the pointer or keyboard moves to a cell, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a cell.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    rsx! {
        PartitionChart { root, label, height, format, highlight: None, on_hover, on_select }
    }
}

/// The icicle and flame graph body. `highlight` is `Some` for a flame graph: rows run
/// bottom-up, branches keep self time, cells colour by name and matches stand out.
#[allow(clippy::many_single_char_names, reason = "x, y, w for geometry read best short")]
#[component]
pub(crate) fn PartitionChart(
    root: ReadSignal<Tree>,
    label: String,
    height: u16,
    format: Option<Callback<f64, String>>,
    highlight: Option<ReadSignal<String>>,
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let flame = highlight.is_some();
    let query = highlight.map(|h| h()).unwrap_or_default();
    let query = query.trim().to_string();
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let plot_h = f64::from(height);
    let mut zoom = use_signal(Vec::<usize>::new);
    let mut active = use_signal(|| None::<usize>);
    let mut marks = use_signal(|| None::<Rc<MountedData>>);
    let tree = Rc::new(root());
    let l = partition::layout(&tree, &zoom(), flame);
    let crumbs = tree.crumbs(&zoom());
    let last = crumbs.len() - 1;
    let up = (last > 0).then(|| crumbs[last - 1].1.clone());
    let current = active().filter(|&i| i < l.cells.len());
    let now = current.and_then(|i| EventDatum::from_node(&tree, &l.cells[i].path));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let go = Callback::new(move |path: Vec<usize>| {
        active.set(None);
        zoom.set(path);
        if let Some(m) = marks() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    });
    // A cell with no colour slot is the muted "Other" colour.
    let fill = move |c: &partition::Cell| {
        if flame {
            colour(partition::name_slot(&c.name))
        } else {
            c.colour.map_or_else(|| colour_or_muted(0, 0), colour)
        }
    };
    let whole = l.cells.first().map_or(0.0, |c| c.value);
    let share = move |v: f64| if whole > 0.0 { v / whole * 100.0 } else { 0.0 };
    let targets: Vec<Option<Vec<usize>>> =
        l.cells.iter().map(|c| if c.depth == 0 { up.clone() } else { c.zoomable.then(|| c.path.clone()) }).collect();
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
    let legend = if flame {
        Vec::new()
    } else {
        tree.node_at(&tree.valid_path(&zoom())).map(partition::legend).unwrap_or_default()
    };
    let scope = if flame { "flame-graph-chart" } else { "icicle-chart" };
    let noun = if flame { "frames" } else { "cells" };
    let count = l.cells.len();
    let key_targets = targets.clone();
    let key_up = up;
    let key_now = now;
    // `y` is only the `data-vertical` flag (1.0 = below), since the style places the tooltip.
    let tip = current.map(|i| {
        let c = &l.cells[i];
        let (x, y, w, rh) = c.rect(l.levels, WIDTH, plot_h, flame);
        let (fx, fy) = ((x + w / 2.0) / WIDTH, if flame { y / plot_h } else { (y + rh).min(plot_h) / plot_h });
        let mut rows = vec![TipRow::new(None, if flame { "Total" } else { "Value" }, fmt(c.value))];
        if flame {
            rows.push(TipRow::new(None, "Self", fmt(c.self_value)));
        }
        rows.push(TipRow::new(None, "Share", format!("{:.1}%", share(c.value))));
        let tip = Tip { x: fx, y: if fy < 0.25 { 1.0 } else { 0.0 }, title: c.name.clone(), rows };
        (tip, fill(c), format!("left: {}; top: {};", pct(fx), pct(fy)))
    });

    rsx! {
        figure { "data-scope": scope, "data-part": "root", "data-searching": (!query.is_empty()).then_some(""),
            Breadcrumb { crumbs: crumbs, go }
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
                    onkeydown: move |e| match zoom_key(&e.key().to_string(), current, count, key_up.is_some()) {
                        Some(ZoomKey::Activate(i)) => {
                            if let (Some(on_select), Some(d)) = (on_select, key_now.clone()) {
                                on_select.call(d);
                            }
                            if let Some(t) = key_targets[i].clone() {
                                e.prevent_default();
                                go.call(t);
                            }
                        }
                        Some(ZoomKey::Up) => {
                            if let Some(p) = key_up.clone() {
                                e.prevent_default();
                                go.call(p);
                            }
                        }
                        Some(ZoomKey::Move(next)) => {
                            e.prevent_default();
                            active.set(next);
                        }
                        None => {}
                    },
                    for (i, c) in l.cells.iter().enumerate() {
                        {
                            let (x, y, w, rh) = c.rect(l.levels, WIDTH, plot_h, flame);
                            let text = (rh >= 16.0).then(|| partition::fit_label(&c.name, w, 6.5)).flatten();
                            let t = targets[i].clone();
                            let (path, tree) = (c.path.clone(), Rc::clone(&tree));
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
                                        if let Some(on_select) = on_select
                                            && let Some(d) = EventDatum::from_node(&tree, &path)
                                        {
                                            on_select.call(d);
                                        }
                                        if let Some(p) = t.clone() {
                                            go.call(p);
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
                if let Some((tip, swatch, style)) = tip {
                    ChartTip { tip, swatch, style }
                }
            }
            PartitionLegend { items: legend }
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
