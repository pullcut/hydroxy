//! `VoronoiTreemapChart`: a tree as nested weighted Voronoi cells inside a circle or a
//! rectangle, each cell's area its share of its parent. Layout comes from
//! `hydroxy_charts::voronoi`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{chord, format_value, nav, voronoi};

use crate::callbacks::{select_active, use_hover};
use crate::plot_kit::colour_or_muted;
use crate::sets_kit::SetsTip;
use crate::sparkline::use_datum;

pub use hydroxy_charts::hierarchy::Tree;
pub use hydroxy_charts::voronoi::Shape;

/// A Voronoi treemap. Cells are coloured by their top-level group, with the group
/// outlines drawn heavier; the larger cells carry their name and value. Hover or touch a
/// cell to read it; focus the chart and use the arrow keys to step through the cells,
/// Home and End to jump, Escape to clear. The same data always draws the same picture.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn VoronoiTreemapChart(
    /// The tree. Branch values come from their children.
    root: ReadSignal<Tree>,
    /// What the chart shows, such as "Cloud spend by service". Used as the accessible name.
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[props(default = 520)]
    width: u16,
    /// Drawing height in px.
    #[props(default = 520)]
    height: u16,
    /// The outline the cells fill.
    #[props(default)]
    shape: Shape,
    /// How many levels to draw, 1 to 4.
    #[props(default = 2)]
    depth: u8,
    /// Formats values for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused cell changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a cell.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let tree = root();
    // The tessellation iterates, so it's kept between hovers and redone only when its inputs change.
    let lay = use_memo(use_reactive((&width, &height, &shape, &depth), move |(width, height, shape, depth)| {
        root.with(|r| voronoi::layout(r, f64::from(width), f64::from(height), shape, usize::from(depth.clamp(1, 4))))
    }));
    let lay = lay.read();
    // Only the leaves are marks; branches show through their children.
    let leaves: Vec<usize> = (0..lay.cells.len()).filter(|&i| lay.cells[i].leaf).collect();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| lay.cells.get(i).is_some_and(|c| c.leaf));
    let datum = use_datum(current.and_then(|i| EventDatum::from_node(&tree, &lay.cells[i].path)));
    use_hover(datum, on_hover);
    let tip = current.and_then(|i| lay.tip(i, &fmt));
    let announcement = tip.as_ref().map(hydroxy_charts::plot::Tip::announce).unwrap_or_default();
    let groups = lay.groups;
    let mut legend: Vec<(String, String)> = lay
        .cells
        .iter()
        .filter(|c| c.depth == 1 && chord::slot(c.colour, groups).is_some())
        .map(|c| (chord::colour(c.colour, groups), c.name.clone()))
        .collect();
    if groups > 6 {
        legend.push((colour_or_muted(0, 0), "Other".to_string()));
    }
    let shape_attr = match shape {
        Shape::Circle => "circle",
        Shape::Rect => "rect",
    };

    rsx! {
        figure {
            "data-scope": "voronoi-treemap-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    "data-shape": shape_attr,
                    view_box: "0 0 {width} {height}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through the cells.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            select_active(datum, on_select);
                        }
                        let at = active().and_then(|a| leaves.iter().position(|&i| i == a));
                        if let Some(next) = nav::list_key(&e.key().to_string(), at, leaves.len()) {
                            e.prevent_default();
                            active.set(next.map(|k| leaves[k]));
                        }
                    },
                    g { "data-part": "cells",
                        for (i , c) in lay.cells.iter().enumerate() {
                            path {
                                key: "{i}",
                                "data-part": "cell",
                                "data-depth": "{c.depth}",
                                "data-leaf": c.leaf.then_some(""),
                                "data-active": (current == Some(i)).then_some(""),
                                d: "{c.d}",
                                fill: chord::colour(c.colour, groups),
                                style: "--i: {c.path.last().copied().unwrap_or(0)}; --d: {c.depth - 1};",
                                onpointerenter: {
                                    let leaf = c.leaf;
                                    move |_| {
                                        if leaf {
                                            active.set(Some(i));
                                        }
                                    }
                                },
                                onpointerdown: {
                                    let leaf = c.leaf;
                                    move |_| {
                                        if leaf {
                                            active.set(Some(i));
                                        }
                                    }
                                },
                                onclick: {
                                    let leaf = c.leaf;
                                    move |_| {
                                        if leaf {
                                            select_active(datum, on_select);
                                        }
                                    }
                                },
                            }
                        }
                    }
                    g { "data-part": "borders",
                        for c in lay.cells.iter().filter(|c| !c.leaf) {
                            path { "data-part": "border", "data-depth": "{c.depth}", d: "{c.d}" }
                        }
                    }
                    path { "data-part": "outline", d: "{lay.outline}" }
                    g { "data-part": "labels",
                        for c in lay.cells.iter().filter(|c| c.leaf && c.label_fits()) {
                            text {
                                "data-part": "label",
                                x: "{c.centroid.0}",
                                y: "{c.centroid.1 - 6.0}",
                                text_anchor: "middle",
                                "{c.name}"
                                tspan { "data-part": "label-value", x: "{c.centroid.0}", dy: "14", {fmt(c.value)} }
                            }
                        }
                    }
                }
                if let Some(tip) = tip {
                    SetsTip { tip }
                }
            }
            if groups > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (c , t) in legend {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {c};" }
                            "{t}"
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
                    }
                }
                tbody {
                    for (name , v) in tree.flatten() {
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
