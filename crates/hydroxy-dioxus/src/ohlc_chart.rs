//! `OhlcChart`: open-high-low-close bars, a range per period with the open ticked to
//! the left and the close to the right. Geometry comes from `hydroxy_charts::ohlc`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::ohlc;

use crate::plot_shell::{PlotShell, pct};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::candle::Candle;
pub use hydroxy_charts::scale::ScaleKind;

/// Price bars, coloured by whether the period closed up or down. Lighter than candles
/// when there are many periods. Hover, touch or focus the plot and use the arrow keys to
/// read a period; Home and End jump, Escape clears.
#[component]
pub fn OhlcChart(
    /// The periods, oldest first.
    candles: ReadSignal<Vec<Candle>>,
    /// One label per period, such as a date. Used on the axis, tooltip and table.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "ACME daily". Used as the accessible name.
    label: String,
    /// Price axis mapping.
    #[props(default)]
    scale: ScaleKind,
    /// Reference lines, bands and labelled points. Y values are prices, x values are period indices.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 260)]
    height: u16,
    /// Formats prices for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus, and `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let layout = ohlc::layout(&candles(), &labels(), scale, &annotations());
    let active = use_signal(|| None::<usize>);
    // Stops skip periods that cannot be drawn, so map the stop back to its candle.
    let now = active().and_then(|stop| {
        let at = layout.candles.iter().enumerate().filter(|(_, g)| g.is_some()).nth(stop).map(|(i, _)| i)?;
        EventDatum::from_candle(&candles.read(), at, labels.read().get(at).map(String::as_str))
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    #[allow(clippy::cast_precision_loss, reason = "period counts are small")]
    let width = 1.0 / layout.candles.len().max(1) as f64;
    #[allow(clippy::cast_precision_loss, reason = "period counts are small")]
    let left = move |i: usize| pct(i as f64 * width);
    rsx! {
        PlotShell { scope: "ohlc-chart", label, hint: "Use the left and right arrow keys to read periods.", height, readout: layout.readout, format, active, datum: Some(datum), on_hover, on_select,
            div { "data-part": "marks", "aria-hidden": "true",
                for (i , g) in layout.candles.iter().enumerate().filter_map(|(i, g)| g.map(|g| (i, g))) {
                    div {
                        key: "{i}",
                        "data-part": "bar",
                        "data-dir": if g.up { "up" } else { "down" },
                        style: "left: {left(i)}; width: {pct(width)}; --i: {i}; --ox-n: {layout.candles.len()};",
                        span { "data-part": "range", style: "bottom: {pct(g.low)}; height: {pct(g.high - g.low)};" }
                        span { "data-part": "open", style: "bottom: {pct(g.open)};" }
                        span { "data-part": "close", style: "bottom: {pct(g.close)};" }
                    }
                }
            }
        }
    }
}
