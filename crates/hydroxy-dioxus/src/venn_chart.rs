//! `VennChart`: two or three sets as circles whose areas match their sizes, with each
//! pairwise overlap solved to the size of the members the sets share. Layout comes from
//! `hydroxy_charts::venn`.

use dioxus::prelude::*;
use hydroxy_charts::{chord, format_value, nav, venn};

use crate::sets_kit::SetsTip;

pub use hydroxy_charts::venn::Intersection;

/// An area-proportional Venn diagram. Every exclusive region carries its member count;
/// hover or touch a region to read it and its share. Focus the chart and use the arrow
/// keys to step through the regions, Home and End to jump, Escape to clear. Only the
/// three largest sets are drawn; for more, use `UpsetChart`.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn VennChart(
    /// Members per exclusive region: each entry counts the members in exactly those sets.
    data: ReadSignal<Vec<Intersection>>,
    /// What the chart shows, such as "Customers by channel". Used as the accessible name.
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[props(default = 520)]
    width: u16,
    /// Drawing height in px.
    #[props(default = 360)]
    height: u16,
    /// Formats counts for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let lay = venn::layout(&data(), f64::from(width), f64::from(height));
    // Keyboard steps through the regions that have room on the page.
    let shown: Vec<usize> = (0..lay.regions.len()).filter(|&i| lay.regions[i].label.is_some()).collect();
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < lay.regions.len());
    let in_active = |set: usize| current.is_some_and(|a| lay.regions[a].sets.contains(&set));
    let tip = current.and_then(|i| lay.tip(i, &fmt));
    let announcement = tip.as_ref().map(hydroxy_charts::plot::Tip::announce).unwrap_or_default();
    let count = lay.circles.len();
    let names: Vec<String> = (0..lay.regions.len()).map(|k| lay.region_name(k)).collect();

    rsx! {
        figure {
            "data-scope": "venn-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {width} {height}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through the regions.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        let at = active().and_then(|a| shown.iter().position(|&i| i == a));
                        if let Some(next) = nav::list_key(&e.key().to_string(), at, shown.len()) {
                            e.prevent_default();
                            active.set(next.map(|k| shown[k]));
                        }
                    },
                    g { "data-part": "circles",
                        for (i , c) in lay.circles.iter().enumerate() {
                            circle {
                                key: "{i}",
                                "data-part": "circle",
                                "data-highlighted": in_active(i).then_some(""),
                                cx: "{c.cx}",
                                cy: "{c.cy}",
                                r: "{c.r}",
                                fill: chord::colour(i, count),
                                style: "--i: {i};",
                            }
                        }
                    }
                    g { "data-part": "regions",
                        for (k , r) in lay.regions.iter().enumerate().filter(|(_, r)| !r.path.is_empty()) {
                            path {
                                key: "{k}",
                                "data-part": "region",
                                "data-active": (current == Some(k)).then_some(""),
                                d: "{r.path}",
                                fill_rule: "evenodd",
                                onpointerenter: move |_| active.set(Some(k)),
                                onpointerdown: move |_| active.set(Some(k)),
                            }
                        }
                    }
                    g { "data-part": "labels",
                        for (k , r , (x , y)) in lay.regions.iter().enumerate().filter_map(|(k, r)| Some((k, r, r.label?))) {
                            text {
                                key: "{k}",
                                "data-part": "region-value",
                                "data-active": (current == Some(k)).then_some(""),
                                "data-empty": (r.count <= 0.0).then_some(""),
                                x: "{x}",
                                y: "{y}",
                                text_anchor: "middle",
                                dominant_baseline: "central",
                                {fmt(r.count)}
                            }
                        }
                        for c in lay.circles.iter() {
                            text {
                                "data-part": "set-name",
                                x: "{c.label.0}",
                                y: "{c.label.1}",
                                text_anchor: c.label.2,
                                dominant_baseline: "central",
                                "{c.name}"
                                tspan { "data-part": "set-size", dx: "6", {fmt(c.size)} }
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
                        th { scope: "col", "Region" }
                        th { scope: "col", "Members" }
                    }
                }
                tbody {
                    for (name , r) in names.iter().zip(lay.regions.iter()) {
                        tr {
                            th { scope: "row", "{name}" }
                            td { {fmt(r.count)} }
                        }
                    }
                }
            }
        }
    }
}
