//! Shared parts for the set and part-of-whole charts (`UpSet`, Venn, Voronoi treemap,
//! nested donut and alluvial): the anchored tooltip.

use dioxus::prelude::*;
use hydroxy_charts::plot::Tip;

/// The tooltip for `tip`, pinned at its anchor in % of the plot.
#[component]
pub(crate) fn SetsTip(tip: Tip) -> Element {
    let (left, top) = (tip.x * 100.0, (1.0 - tip.y) * 100.0);
    rsx! {
        div {
            "data-part": "tooltip",
            "data-side": tip.side(),
            "data-vertical": tip.vertical(),
            style: "left: {left:.2}%; top: {top:.2}%;",
            "aria-hidden": "true",
            div { "data-part": "tooltip-label", "{tip.title}" }
            ul {
                for r in tip.rows.iter() {
                    li { "data-part": "tooltip-row",
                        span { "data-part": "tooltip-name",
                            if let Some(c) = &r.colour {
                                span { "data-part": "swatch", style: "background: {c};" }
                            }
                            "{r.name}"
                        }
                        span { "data-part": "tooltip-value", "{r.value}" }
                    }
                }
            }
        }
    }
}
