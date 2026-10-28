//! `<OhlcChart>`: open-high-low-close bars, a range per period with the open ticked to
//! the left and the close to the right. Geometry comes from `hydroxy_charts::ohlc`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::ohlc;
use leptos::prelude::*;

use crate::plot_kit::pct;
use crate::plot_shell::PlotShell;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::candle::Candle;
pub use hydroxy_charts::scale::ScaleKind;

/// Price bars, coloured by whether the period closed up or down. Lighter than candles
/// when there are many periods. Hover, touch or focus the plot and use the arrow keys to
/// read a period; Home and End jump, Escape clears.
#[component]
pub fn OhlcChart(
    /// The periods, oldest first.
    #[prop(into)]
    candles: Signal<Vec<Candle>>,
    /// One label per period, such as a date. Used on the axis, tooltip and table.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows, such as "ACME daily". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Price axis mapping.
    #[prop(optional)]
    scale: ScaleKind,
    /// Reference lines, bands and labelled points. Y values are prices, x values are period indices.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 260)]
    height: u16,
    /// Formats prices for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let layout =
        Memo::new(move |_| candles.with(|c| labels.with(|l| annotations.with(|a| ohlc::layout(c, l, scale, a)))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);
    // Stops skip periods that cannot be drawn, so map the stop back to its candle.
    let datum = Signal::derive(move || {
        let stop = active.get()?;
        let at =
            layout.with(|l| l.candles.iter().enumerate().filter(|(_, g)| g.is_some()).nth(stop).map(|(i, _)| i))?;
        candles.with(|c| labels.with(|l| EventDatum::from_candle(c, at, l.get(at).map(String::as_str))))
    });

    view! {
        <PlotShell scope="ohlc-chart" label hint="Use the left and right arrow keys to read periods." height readout format active datum=Some(datum) on_hover on_select>
            <div data-part="marks" aria-hidden="true">
                {move || {
                    let bars = layout.with(|l| l.candles.clone());
                    #[allow(clippy::cast_precision_loss, reason = "period counts are small")]
                    let width = 1.0 / bars.len().max(1) as f64;
                    bars.into_iter()
                        .enumerate()
                        .filter_map(|(i, g)| g.map(|g| (i, g)))
                        .map(|(i, g)| {
                            #[allow(clippy::cast_precision_loss, reason = "period counts are small")]
                            let left = i as f64 * width;
                            view! {
                                <div
                                    data-part="bar"
                                    data-dir=if g.up { "up" } else { "down" }
                                    style=format!("left: {}; width: {}; --i: {};", pct(left), pct(width), i.min(40))
                                >
                                    <span data-part="range" style=format!("bottom: {}; height: {};", pct(g.low), pct(g.high - g.low))></span>
                                    <span data-part="open" style=format!("bottom: {};", pct(g.open))></span>
                                    <span data-part="close" style=format!("bottom: {};", pct(g.close))></span>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </PlotShell>
    }
}
