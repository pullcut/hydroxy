//! `<DepthChart>`: an order book as cumulative bid and ask step areas meeting at the
//! mid price. Geometry comes from `hydroxy_charts::depth`.

use hydroxy_charts::depth;
use hydroxy_charts::event::Datum as EventDatum;
use leptos::prelude::*;

use crate::plot_kit::{colour, pct};
use crate::plot_shell::PlotShell;

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
    /// Called when the pointer or keyboard moves to a price level, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a price level.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| hydroxy_charts::format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| bids.with(|b| asks.with(|a| annotations.with(|n| depth::layout(b, a, n)))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let mut all: Vec<(Level, bool)> = bids
            .with(|b| depth::cumulative(b, true))
            .into_iter()
            .map(|(l, _)| (l, true))
            .chain(asks.with(|a| depth::cumulative(a, false)).into_iter().map(|(l, _)| (l, false)))
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

    view! {
        <PlotShell scope="depth-chart" label hint="Use the left and right arrow keys to read price levels." height readout format active datum=Some(datum) on_hover on_select>
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
