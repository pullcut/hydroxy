//! `RenkoChart`: fixed-size bricks laid each time price moves a full brick, ignoring
//! time, so trends and reversals stand out from noise. Bricks come from `oxide_charts::renko`.

use dioxus::prelude::*;
use oxide_charts::renko;

use crate::plot_shell::{PlotShell, pct};

pub use oxide_charts::annotate::Annotation;

/// Rising and falling bricks, one column each. Hover, touch or focus the plot and use the
/// arrow keys to read a brick; Home and End jump, Escape clears.
#[component]
pub fn RenkoChart(
    /// Prices in time order, such as daily closes. Non-finite prices are skipped.
    prices: ReadSignal<Vec<f64>>,
    /// Optional label per price, such as a date, naming the price that completed each brick.
    #[props(default)]
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "ACME closes". Used as the accessible name.
    label: String,
    /// Brick size in price units. Default: about a twentieth of the price range, rounded.
    brick_size: Option<f64>,
    /// Reference lines, bands and labelled points. Y values are prices, x values are brick indices.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the legend.
    #[props(default = 240)]
    height: u16,
    /// Formats prices for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let layout = renko::layout(&prices(), &labels(), brick_size, &annotations());
    let active = use_signal(|| None::<usize>);
    #[allow(clippy::cast_precision_loss, reason = "brick counts are small")]
    let width = 1.0 / layout.bricks.len().max(1) as f64;
    #[allow(clippy::cast_precision_loss, reason = "brick counts are small")]
    let left = move |i: usize| pct(i as f64 * width);
    rsx! {
        PlotShell { scope: "renko-chart", label, hint: "Use the left and right arrow keys to read bricks.", height, readout: layout.readout, format, active,
            div { "data-part": "marks", "aria-hidden": "true",
                for (i , b) in layout.bricks.iter().enumerate() {
                    span {
                        key: "{i}",
                        "data-part": "brick",
                        "data-dir": if b.up { "up" } else { "down" },
                        style: "left: {left(i)}; width: {pct(width)}; bottom: {pct(b.bottom)}; height: {pct(b.top - b.bottom)}; --i: {i.min(40)};",
                    }
                }
            }
        }
    }
}
