//! `ViolinChart`: a mirrored density per group with an optional inner box plot.
//! Shapes come from `hydroxy_charts::density::violin`, box stats from `hydroxy_charts::boxplot`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{density, probe};

use crate::density_chart::curves;
use crate::plot_shell::{PlotShell, pct};
use crate::sparkline::use_datum;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;

/// Groups side by side, each drawn as its density mirrored about a centre line, with the
/// median marked. Hover or touch a violin to read its numbers; focus the plot and use
/// the arrow keys to move between groups, Home and End to jump, Escape to clear.
#[component]
pub fn ViolinChart(
    /// The groups, left to right, each with its raw values.
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Deploy time by team". Used as the accessible name.
    label: String,
    /// Draw the quartile box and whiskers inside each violin.
    #[props(default = true)]
    inner_box: bool,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points. Y values are data values, x values are group indices.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 260)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused violin changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a violin.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let all = groups();
    let layout = density::violin(&all, bandwidth, &annotations());
    let active = use_signal(|| None::<usize>);
    let n = layout.boxes.len();
    let datum = use_datum(active().and_then(|g| {
        let name = all.get(g)?.label.clone();
        let median = layout.boxes.get(g).and_then(Option::as_ref).map_or(f64::NAN, |b| b.stats.median);
        Some(EventDatum::category(g, name, median))
    }));
    rsx! {
        PlotShell { scope: "violin-chart", label, hint: "Use the left and right arrow keys to move between groups.", height, readout: layout.readout, format, active, datum, on_hover, on_select,
            {curves(&layout.shapes)}
            div { "data-part": "boxes", "aria-hidden": "true",
                for (i , b) in layout.boxes.iter().enumerate().filter_map(|(i, b)| b.as_ref().map(|b| (i, b))) {
                    div { key: "{i}", "data-part": "inner", style: "left: {pct(probe::band(i, n))}; --i: {i};",
                        if inner_box {
                            span { "data-part": "whisker", style: "bottom: {pct(b.low)}; height: {pct(b.high - b.low)};" }
                            span { "data-part": "box", style: "bottom: {pct(b.q1)}; height: {pct(b.q3 - b.q1)};" }
                        }
                        span { "data-part": "median", style: "bottom: {pct(b.median)};" }
                    }
                }
            }
        }
    }
}
