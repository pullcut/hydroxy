//! `AlluvialChart`: how members move between categories across ordered steps. Each
//! step is a column of stacked categories, always in the same order; ribbons carry the
//! members from one step's category to the next's. Layout comes from
//! `hydroxy_charts::alluvial`.

use dioxus::prelude::*;
use hydroxy_charts::{alluvial, format_value, nav};

use crate::sets_kit::SetsTip;

pub use hydroxy_charts::alluvial::Cohort;

/// Room at the top for the step names.
const HEAD: f64 = 26.0;

/// An alluvial diagram. Colour follows the category, so a ribbon shows where its
/// members came from. Hover or touch a category to light its ribbons, or a ribbon to read
/// how many moved; focus the chart and use the arrow keys to step through the categories
/// and then the ribbons, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn AlluvialChart(
    /// Groups of members that share a path, one category per step. Categories stack in
    /// order of first appearance; paths shorter than the longest are ignored.
    data: ReadSignal<Vec<Cohort>>,
    /// Step names, such as months, in order.
    steps: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Customers by plan, quarter to quarter". Used as the accessible name.
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[props(default = 720)]
    width: u16,
    /// Drawing height in px.
    #[props(default = 360)]
    height: u16,
    /// Formats counts for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    let lay = alluvial::layout(&data(), box_w, box_h, HEAD, 12.0, 10.0);
    let step_names = steps();
    let step_name = |t: usize| step_names.get(t).cloned().unwrap_or_else(|| format!("Step {}", t + 1));
    // Marks are the nodes first, then the ribbons.
    let mut active = use_signal(|| None::<usize>);
    let n = lay.nodes.len();
    let marks = n + lay.ribbons.len();
    let current = active().filter(|&m| m < marks);
    let lit = current.map(|m| lay.lit(m));
    let node_lit = |i: usize| lit.as_ref().is_some_and(|(x, _)| x.get(i).copied().unwrap_or(false)).then_some("");
    let ribbon_lit = |k: usize| lit.as_ref().is_some_and(|(_, x)| x.get(k).copied().unwrap_or(false)).then_some("");
    let tip = current.and_then(|m| lay.tip(m, &step_names, box_w, box_h, &fmt));
    let announcement = tip.as_ref().map(hydroxy_charts::plot::Tip::announce).unwrap_or_default();
    let last = lay.steps.saturating_sub(1);
    let heads: Vec<(f64, &'static str, String)> = lay
        .columns
        .iter()
        .enumerate()
        .map(|(t, &x)| match t {
            0 => (x, "start", step_name(t)),
            t if t == last => (x + lay.node_width, "end", step_name(t)),
            _ => (x + lay.node_width / 2.0, "middle", step_name(t)),
        })
        .collect();

    rsx! {
        figure {
            "data-scope": "alluvial-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {width} {height}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through the categories and then the ribbons.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), marks) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    g { "data-part": "steps",
                        for (x , anchor , name) in heads {
                            text { "data-part": "step", x: "{x}", y: "12", text_anchor: anchor, "{name}" }
                        }
                    }
                    g { "data-part": "flow",
                        g { "data-part": "ribbons",
                            for (k , r) in lay.ribbons.iter().enumerate() {
                                path {
                                    key: "{k}",
                                    "data-part": "ribbon",
                                    "data-highlighted": ribbon_lit(k),
                                    d: "{r.path}",
                                    fill: lay.colour(lay.nodes[r.source].category),
                                    onpointerenter: move |_| active.set(Some(n + k)),
                                    onpointerdown: move |_| active.set(Some(n + k)),
                                }
                            }
                        }
                        g { "data-part": "nodes",
                            for (i , node) in lay.nodes.iter().enumerate() {
                                rect {
                                    key: "{i}",
                                    "data-part": "node",
                                    "data-highlighted": node_lit(i),
                                    "data-selected": (current == Some(i)).then_some(""),
                                    x: "{node.x}",
                                    y: "{node.y}",
                                    width: "{lay.node_width}",
                                    height: "{node.height.max(1.0)}",
                                    rx: "2",
                                    fill: lay.colour(node.category),
                                    onpointerenter: move |_| active.set(Some(i)),
                                    onpointerdown: move |_| active.set(Some(i)),
                                }
                            }
                        }
                    }
                    g { "data-part": "labels",
                        for node in lay.nodes.iter().filter(|node| node.height >= 14.0) {
                            text {
                                "data-part": "node-label",
                                x: if node.step == last && last > 0 { node.x - 6.0 } else { node.x + lay.node_width + 6.0 },
                                y: "{node.y + node.height / 2.0}",
                                text_anchor: if node.step == last && last > 0 { "end" } else { "start" },
                                dominant_baseline: "middle",
                                "{lay.categories[node.category]}"
                                tspan { "data-part": "node-value", dx: "6", {fmt(node.value)} }
                            }
                        }
                    }
                }
                if let Some(tip) = tip {
                    SetsTip { tip }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Steps" }
                        th { scope: "col", "From" }
                        th { scope: "col", "To" }
                        th { scope: "col", "Members" }
                    }
                }
                tbody {
                    for r in lay.ribbons.iter() {
                        tr {
                            td { "{step_name(lay.nodes[r.source].step)} → {step_name(lay.nodes[r.target].step)}" }
                            td { "{lay.categories[lay.nodes[r.source].category]}" }
                            td { "{lay.categories[lay.nodes[r.target].category]}" }
                            td { {fmt(r.value)} }
                        }
                    }
                }
            }
        }
    }
}
