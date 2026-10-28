//! `<RenkoChart>`: fixed-size bricks laid each time price moves a full brick, ignoring
//! time, so trends and reversals stand out from noise. Bricks come from `hydroxy_charts::renko`.

use hydroxy_charts::renko;
use leptos::prelude::*;

use crate::plot_shell::{PlotShell, pct};

pub use hydroxy_charts::annotate::Annotation;

/// Rising and falling bricks, one column each. Hover, touch or focus the plot and use the
/// arrow keys to read a brick; Home and End jump, Escape clears.
#[component]
pub fn RenkoChart(
    /// Prices in time order, such as daily closes. Non-finite prices are skipped.
    #[prop(into)]
    prices: Signal<Vec<f64>>,
    /// Optional label per price, such as a date, naming the price that completed each brick.
    #[prop(optional, into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows, such as "ACME closes". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Brick size in price units. Default: about a twentieth of the price range, rounded.
    #[prop(optional, into)]
    brick_size: Option<f64>,
    /// Reference lines, bands and labelled points. Y values are prices, x values are brick indices.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the legend.
    #[prop(default = 240)]
    height: u16,
    /// Formats prices for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let layout =
        Memo::new(move |_| prices.with(|p| labels.with(|l| annotations.with(|a| renko::layout(p, l, brick_size, a)))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);

    view! {
        <PlotShell scope="renko-chart" label hint="Use the left and right arrow keys to read bricks." height readout format active>
            <div data-part="marks" aria-hidden="true">
                {move || {
                    let bricks = layout.with(|l| l.bricks.clone());
                    #[allow(clippy::cast_precision_loss, reason = "brick counts are small")]
                    let width = 1.0 / bricks.len().max(1) as f64;
                    bricks
                        .into_iter()
                        .enumerate()
                        .map(|(i, b)| {
                            #[allow(clippy::cast_precision_loss, reason = "brick counts are small")]
                            let left = i as f64 * width;
                            view! {
                                <span
                                    data-part="brick"
                                    data-dir=if b.up { "up" } else { "down" }
                                    style=format!(
                                        "left: {}; width: {}; bottom: {}; height: {}; --i: {};",
                                        pct(left),
                                        pct(width),
                                        pct(b.bottom),
                                        pct(b.top - b.bottom),
                                        i.min(40),
                                    )
                                ></span>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </PlotShell>
    }
}
