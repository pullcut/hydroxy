//! `DepthChart`: an order book as cumulative bid and ask step areas meeting at the
//! mid price. Geometry comes from `hydroxy_charts::depth`.

use dioxus::prelude::*;
use hydroxy_charts::{depth, format_value};

use crate::plot_shell::{PlotShell, colour, pct};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::depth::Level;

/// Bids step down to the left of the spread and asks step up to the right, each height
/// the total size available up to that price. Hover, touch or focus the plot and use
/// the arrow keys to read each level; Home and End jump, Escape clears.
#[component]
pub fn DepthChart(
    /// Bid levels, any order.
    bids: ReadSignal<Vec<Level>>,
    /// Ask levels, any order.
    asks: ReadSignal<Vec<Level>>,
    /// What the chart shows, such as "BTC-USD order book". Used as the accessible name.
    label: String,
    /// Reference lines, bands and labelled points. X values are prices, y values are cumulative sizes.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 240)]
    height: u16,
    /// Formats prices for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let layout = depth::layout(&bids(), &asks(), &annotations());
    let active = use_signal(|| None::<usize>);
    let sides = [(layout.bids, depth::BID, "bid"), (layout.asks, depth::ASK, "ask")];
    rsx! {
        PlotShell { scope: "depth-chart", label, hint: "Use the left and right arrow keys to read price levels.", height, readout: layout.readout, format, active,
            svg { "data-part": "marks", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                for (s , c , side) in sides {
                    g { key: "{side}", "data-part": "curve", "data-side": side,
                        path { "data-part": "area", d: "{s.area}", fill: colour(c) }
                        path { "data-part": "line", d: "{s.line}", stroke: colour(c) }
                    }
                }
            }
            if let Some((price , at)) = layout.mid {
                div { "data-part": "mid", style: "left: {pct(at)};", "aria-hidden": "true",
                    span { "data-part": "mid-label", "Mid {fmt(price)}" }
                }
            }
        }
    }
}
