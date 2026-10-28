//! `BubblePackChart`: a tree as nested circles, each leaf's area its value. Click a
//! circle with children to zoom into it. Layout comes from `hydroxy_charts::pack`.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, nav, pack, partition};
use std::rc::Rc;

pub use hydroxy_charts::hierarchy::Tree;

fn colour(slot: Option<usize>) -> String {
    slot.map_or_else(|| "var(--chart-muted)".to_string(), |s| format!("var(--chart-{})", s + 1))
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Circle packing. Hover, touch or focus the chart and use the arrow keys to read each
/// circle; click one with children, or press Enter on it, to zoom in. Escape, the
/// outer circle or the breadcrumb zoom back out.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
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
    let mut go = move |path: Vec<usize>| {
        active.set(None);
        zoom.set(path);
        if let Some(m) = marks() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
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
    let labels: Vec<(f64, f64, bool, String)> = bubbles
        .iter()
        .filter(|b| b.depth > 0 && b.r > 22.0 && (!b.zoomable || b.depth == 1))
        .filter_map(|b| {
            Some((
                b.x,
                if b.zoomable { b.y - b.r + 14.0 } else { b.y },
                b.zoomable,
                partition::fit_label(&b.name, b.r * 1.8, 6.5)?,
            ))
        })
        .collect();
    let tip = current.map(|i| bubbles[i].clone());
    let count = bubbles.len();
    let key_targets = targets.clone();
    let key_up = up.clone();

    rsx! {
        figure { "data-scope": "bubble-pack-chart", "data-part": "root", style: "max-width: {size}px;",
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
                    view_box: "0 0 {size} {size}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to move between circles, Enter to zoom in, Escape to zoom out.",
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
                            fill: colour(b.colour),
                            style: "--i: {i.min(40)};",
                            onpointerenter: move |_| active.set(Some(i)),
                            onpointerdown: move |_| active.set(Some(i)),
                            onclick: {
                                let t = targets[i].clone();
                                move |_| {
                                    if let Some(p) = t.clone() {
                                        go(p);
                                    }
                                }
                            },
                        }
                    }
                    for (x, y, zoomable, text) in labels {
                        text { "data-part": "label", "data-leaf": (!zoomable).then_some(""), x: "{x:.2}", y: "{y:.2}", text_anchor: "middle", dominant_baseline: "central", "{text}" }
                    }
                }
                if let Some(b) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if b.x / px > 0.5 { "left" } else { "right" },
                        "data-vertical": if (b.y - b.r).max(0.0) / px < 0.25 { "below" } else { "above" },
                        style: "left: {pct(b.x / px)}; top: {pct((b.y - b.r).max(0.0) / px)};",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {colour(b.colour)};" }
                            "{b.name}"
                        }
                        ul {
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Value" }
                                span { "data-part": "tooltip-value", {fmt(b.value)} }
                            }
                            li { "data-part": "tooltip-row",
                                span { "data-part": "tooltip-name", "Share" }
                                span { "data-part": "tooltip-value", "{share(b.value):.1}%" }
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
