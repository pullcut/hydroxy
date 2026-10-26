//! `SunburstChart`: a tree as concentric rings, each arc's sweep its share of the
//! whole. Click an arc to zoom into it, the centre to climb out. Layout comes from
//! `oxide_charts::sunburst`.

use dioxus::prelude::*;
use oxide_charts::{format_value, sunburst};
use std::rc::Rc;

pub use oxide_charts::hierarchy::Tree;

/// A sunburst. Arcs with children can be clicked, or focused and activated with Enter
/// or Space, to zoom in. The centre shows the hovered arc, or the current level, and
/// zooms back out when activated.
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
    /// Formats values for the centre and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let s = f64::from(size);
    // The centre disc matches the layout's hole: 26% of the outer radius.
    let hole_pct = 2.0 * (s / 2.0 - 2.0) * 0.26 / s * 100.0;
    let mut zoom = use_signal(Vec::<usize>::new);
    let mut hovered = use_signal(|| None::<usize>);
    let mut plot = use_signal(|| None::<Rc<MountedData>>);
    let tree = root();
    let lay = sunburst::layout(&tree, &zoom(), s, usize::from(depth.clamp(1, 4)));
    let crumbs = tree.crumbs(&zoom());
    let last = crumbs.len() - 1;
    let mut go = move |path: Vec<usize>| {
        hovered.set(None);
        zoom.set(path);
        if let Some(m) = plot() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
    let up = (last > 0).then(|| crumbs[last - 1].1.clone());
    let (name, value) = match hovered().and_then(|i| lay.arcs.get(i)) {
        Some(a) => (a.name.clone(), a.value),
        None => (crumbs[last].0.clone(), tree.node_at(&tree.valid_path(&zoom())).map_or(0.0, Tree::total)),
    };
    let top = last == 0;

    rsx! {
        figure { "data-scope": "sunburst-chart", "data-part": "root", "aria-label": "{label}", style: "max-width: {size}px;",
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
            div { "data-part": "plot", tabindex: "-1", onmounted: move |e| plot.set(Some(e.data())),
                svg { view_box: "0 0 {size} {size}", role: "group", "aria-label": "{label}",
                    for (i, a) in lay.arcs.iter().enumerate() {
                        path {
                            key: "{a.path:?}",
                            "data-part": "arc",
                            "data-depth": "{a.depth}",
                            "data-leaf": (!a.zoomable).then_some(""),
                            d: "{a.path_d}",
                            fill: "var(--chart-{a.colour % 6 + 1})",
                            tabindex: "0",
                            role: "button",
                            "aria-disabled": (!a.zoomable).then_some("true"),
                            "aria-label": format!(
                                "{}: {}, {:.0}%{}",
                                a.name,
                                fmt(a.value),
                                a.share * 100.0,
                                if a.zoomable { ", press Enter to zoom in" } else { "" },
                            ),
                            onpointerenter: move |_| hovered.set(Some(i)),
                            onpointerleave: move |_| hovered.set(None),
                            onfocus: move |_| hovered.set(Some(i)),
                            onblur: move |_| hovered.set(None),
                            onclick: {
                                let (path, zoomable) = (a.path.clone(), a.zoomable);
                                move |_| {
                                    if zoomable {
                                        go(path.clone());
                                    }
                                }
                            },
                            onkeydown: {
                                let (path, zoomable) = (a.path.clone(), a.zoomable);
                                move |ev: Event<KeyboardData>| {
                                    if zoomable && (ev.key() == Key::Enter || ev.key() == Key::Character(" ".into())) {
                                        ev.prevent_default();
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
