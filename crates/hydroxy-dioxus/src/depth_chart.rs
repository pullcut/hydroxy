//! `DepthChart`: an order book as cumulative bid and ask step areas meeting at the
//! mid price. Geometry comes from `hydroxy_charts::depth`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
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
    /// Called when the pointer or keyboard moves to a price level, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a price level.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let layout = depth::layout(&bids(), &asks(), &annotations());
    let active = use_signal(|| None::<usize>);
    let now = (on_hover.is_some() || on_select.is_some()).then_some(active()).flatten().and_then(|i| {
        let mut all: Vec<(Level, bool)> = depth::cumulative(&bids.read(), true)
            .into_iter()
            .map(|(l, _)| (l, true))
            .chain(depth::cumulative(&asks.read(), false).into_iter().map(|(l, _)| (l, false)))
            .collect();
        all.sort_by(|x, y| x.0.price.total_cmp(&y.0.price));
        let (level, bid) = *all.get(i)?;
        Some(EventDatum {
            series: usize::from(!bid),
            index: i,
            label: Some(fmt(level.price)),
            value: level.size,
            node: None,
        })
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    let sides = [(layout.bids, depth::BID, "bid"), (layout.asks, depth::ASK, "ask")];
    rsx! {
        PlotShell { scope: "depth-chart", label, hint: "Use the left and right arrow keys to read price levels.", height, readout: layout.readout, format, active, datum: Some(datum), on_hover, on_select,
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
