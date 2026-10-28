//! `SunburstChart`: a tree as concentric rings, each arc's sweep its share of the
//! whole. Click an arc to zoom into it, the centre to climb out. Layout comes from
//! `hydroxy_charts::sunburst`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, sunburst};
use std::rc::Rc;

use crate::callbacks::{select_active, use_hover};
use crate::hierarchy_kit::{Breadcrumb, MarkLegend, drill};
use crate::plot_kit::{ChartTip, colour};
use crate::sparkline::use_datum;

pub use hydroxy_charts::hierarchy::Tree;

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
    /// Called when the hovered or focused arc changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on an arc.
    on_select: Option<EventHandler<EventDatum>>,
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
    let datum = use_datum(current.and_then(|i| EventDatum::from_node(&tree, &lay.arcs[i].path)));
    use_hover(datum, on_hover);
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
    let top_arcs: Vec<(usize, String, usize)> =
        lay.arcs.iter().enumerate().filter(|(_, a)| a.depth == 1).map(|(i, a)| (i, a.name.clone(), a.colour)).collect();
    let total = tree.total();

    rsx! {
        figure {
            "data-scope": "sunburst-chart",
            "data-part": "root",
            "aria-label": "{label}",
            "data-active": current.map(|_| ""),
            style: "max-width: {size}px;",
            Breadcrumb { crumbs: crumbs.clone(), go: Callback::new(go) }
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through arcs, Enter to zoom in, Backspace to zoom out.",
                onmounted: move |e| plot.set(Some(e.data())),
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        select_active(datum, on_select);
                    }
                    let mut enter = go;
                    let up_path = up_key.clone();
                    let handled = drill(
                        &e.key().to_string(),
                        active,
                        &zoomable,
                        top,
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
                svg { view_box: "0 0 {size} {size}", "aria-hidden": "true",
                    for (i, a) in lay.arcs.iter().enumerate() {
                        path {
                            key: "{a.path:?}",
                            "data-part": "arc",
                            "data-depth": "{a.depth}",
                            "data-leaf": (!a.zoomable).then_some(""),
                            "data-active": (current == Some(i)).then_some(""),
                            d: "{a.path_d}",
                            fill: colour(a.colour),
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: {
                                let (path, zoomable) = (a.path.clone(), a.zoomable);
                                move |_| {
                                    select_active(datum, on_select);
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
                    ChartTip {
                        tip: Tip {
                            x: ax,
                            y: ay,
                            title: arc.name.clone(),
                            rows: vec![
                                TipRow::new(None, "Value", fmt(arc.value)),
                                TipRow::new(None, format!("Share of {level_name}"), format!("{:.0}%", arc.share * 100.0)),
                            ],
                        },
                        swatch: colour(arc.colour),
                        anchored: true,
                        side: if ax > 0.5 { "right" } else { "left" },
                        style: format!("left: {:.3}%; top: {:.3}%;", ax * 100.0, ay * 100.0),
                    }
                }
            }
            MarkLegend { items: top_arcs, active }
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
