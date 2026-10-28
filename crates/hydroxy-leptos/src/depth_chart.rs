//! `<DepthChart>`: an order book as cumulative bid and ask step areas meeting at the
//! mid price. Geometry comes from `hydroxy_charts::depth`.

use leptos::prelude::*;
use hydroxy_charts::depth;

use crate::plot_shell::{PlotShell, colour, pct};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::depth::Level;

/// Bids step down to the left of the spread and asks step up to the right, each height
/// the total size available up to that price. Hover, touch or focus the plot and use
/// the arrow keys to read each level; Home and End jump, Escape clears.
#[component]
pub fn DepthChart(
    /// Bid levels, any order.
    #[prop(into)]
    bids: Signal<Vec<Level>>,
    /// Ask levels, any order.
    #[prop(into)]
    asks: Signal<Vec<Level>>,
    /// What the chart shows, such as "BTC-USD order book". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Reference lines, bands and labelled points. X values are prices, y values are cumulative sizes.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 240)]
    height: u16,
    /// Formats prices for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| hydroxy_charts::format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| bids.with(|b| asks.with(|a| annotations.with(|n| depth::layout(b, a, n)))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);

    view! {
        <PlotShell scope="depth-chart" label hint="Use the left and right arrow keys to read price levels." height readout format active>
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {move || {
                    let l = layout.get();
                    [(l.bids, depth::BID, "bid"), (l.asks, depth::ASK, "ask")]
                        .into_iter()
                        .map(|(s, c, side)| {
                            view! {
                                <g data-part="curve" data-side=side>
                                    <path data-part="area" d=s.area fill=colour(c)></path>
                                    <path data-part="line" d=s.line stroke=colour(c)></path>
                                </g>
                            }
                        })
                        .collect_view()
                }}
            </svg>
            {move || {
                layout
                    .with(|l| l.mid)
                    .map(|(price, at)| {
                        view! {
                            <div data-part="mid" style=format!("left: {};", pct(at)) aria-hidden="true">
                                <span data-part="mid-label">{format!("Mid {}", fmt(price))}</span>
                            </div>
                        }
                    })
            }}
        </PlotShell>
    }
}
