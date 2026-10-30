//! `BubblePackChart`: a tree as nested circles, each leaf's area its value. Click a
//! circle with children to zoom into it. Layout comes from `hydroxy_charts::pack`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow, colour_or_muted};
use hydroxy_charts::{format_value, pack, partition};
use std::rc::Rc;

use crate::hierarchy_kit::{Breadcrumb, PartitionLegend, ZoomKey, zoom_key};
use crate::plot_kit::{ChartTip, colour, pct};

pub use hydroxy_charts::hierarchy::Tree;

/// Circle packing. Hover, touch or focus the chart and use the arrow keys to read each
/// circle; click one with children, or press Enter on it, to zoom in. Escape, the
/// outer circle or the breadcrumb zoom back out.
#[component]
pub fn BubblePackChart(
    /// The tree. Branch values come from their children.
    root: ReadSignal<Tree>,
    /// What the chart shows, such as "Disk usage by folder". Used as the accessible name.
    label: String,
    /// Diameter in px; the chart scales down to fit its container.
    #[props(default = 480)]
    size: u16,
    /// Formats values for the tooltip, labels and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let px = f64::from(size);
    let mut zoom = use_signal(Vec::<usize>::new);
    let mut active = use_signal(|| None::<usize>);
    let mut marks = use_signal(|| None::<Rc<MountedData>>);
    let tree = root();
    let bubbles = pack::layout(&tree, &zoom(), px, 3.0);
    let crumbs = tree.crumbs(&zoom());
    let last = crumbs.len() - 1;
    let up = (last > 0).then(|| crumbs[last - 1].1.clone());
    let current = active().filter(|&i| i < bubbles.len());
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.and_then(|i| EventDatum::from_node(&tree, &bubbles[i].path))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let go = Callback::new(move |path: Vec<usize>| {
        active.set(None);
        zoom.set(path);
        if let Some(m) = marks() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    });
    // A bubble with no colour slot is the muted "Other" colour.
    let fill = |slot: Option<usize>| slot.map_or_else(|| colour_or_muted(0, 0), colour);
    // Activating a circle zooms into it; activating the outer circle climbs out.
    let targets: Vec<Option<Vec<usize>>> =
        bubbles.iter().map(|b| if b.depth == 0 { up.clone() } else { b.zoomable.then(|| b.path.clone()) }).collect();
    let whole = bubbles.first().map_or(0.0, |b| b.value);
    let share = move |v: f64| if whole > 0.0 { v / whole * 100.0 } else { 0.0 };
    let announcement = current
        .map(|i| {
            let b = &bubbles[i];
            let hint = match (b.depth, b.zoomable) {
                (0, _) if up.is_some() => ", press Enter to zoom out",
                (0, _) => "",
                (_, true) => ", press Enter to zoom in",
                _ => "",
            };
            format!("{}: {}, {:.0}% of {}{hint}", b.name, fmt(b.value), share(b.value), bubbles[0].name)
        })
        .unwrap_or_default();
    let legend = tree.node_at(&tree.valid_path(&zoom())).map(partition::legend).unwrap_or_default();
    let labels =
        bubbles.iter().filter(|b| b.depth > 0 && b.r > 22.0 && (!b.zoomable || b.depth == 1)).filter_map(|b| {
            Some((
                b.x,
                if b.zoomable { b.y - b.r + 14.0 } else { b.y },
                b.zoomable,
                partition::fit_label(&b.name, b.r * 1.8, 6.5)?,
            ))
        });
    // `y` is only the `data-vertical` flag (1.0 = below), since the style places the tooltip.
    let tip = current.map(|i| {
        let b = &bubbles[i];
        let top = (b.y - b.r).max(0.0) / px;
        let rows = vec![
            TipRow::new(None, "Value", fmt(b.value)),
            TipRow::new(None, "Share", format!("{:.1}%", share(b.value))),
        ];
        let tip = Tip { x: b.x / px, y: if top < 0.25 { 1.0 } else { 0.0 }, title: b.name.clone(), rows };
        (tip, fill(b.colour), format!("left: {}; top: {};", pct(b.x / px), pct(top)))
    });
    let count = bubbles.len();
    let key_targets = targets.clone();
    let key_up = up;

    rsx! {
        figure { "data-scope": "bubble-pack-chart", "data-part": "root", style: "max-width: {size}px;",
            Breadcrumb { crumbs: crumbs, go }
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {size} {size}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to move between circles, Enter to zoom in, Escape to zoom out.",
                    onmounted: move |e| marks.set(Some(e.data())),
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| match zoom_key(&e.key().to_string(), current, count, key_up.is_some()) {
                        Some(ZoomKey::Activate(i)) => {
                            crate::callbacks::select_active(Some(datum), on_select);
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
                    for (i, b) in bubbles.iter().enumerate() {
                        circle {
                            key: "{b.path:?}",
                            "data-part": "bubble",
                            "data-depth": "{b.depth}",
                            "data-leaf": (!b.zoomable).then_some(""),
                            "data-active": (current == Some(i)).then_some(""),
                            cx: "{b.x:.2}",
                            cy: "{b.y:.2}",
                            r: "{b.r:.2}",
                            fill: fill(b.colour),
                            style: "--i: {i}; --ox-n: {bubbles.len()};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: {
                                let t = targets[i].clone();
                                move |_| {
                                    active.set(Some(i));
                                    crate::callbacks::select_active(Some(datum), on_select);
                                    if let Some(p) = t.clone() {
                                        go.call(p);
                                    }
                                }
                            },
                        }
                    }
                    for (x, y, zoomable, text) in labels {
                        text { "data-part": "label", "data-leaf": (!zoomable).then_some(""), x: "{x:.2}", y: "{y:.2}", text_anchor: "middle", dominant_baseline: "central", "{text}" }
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
