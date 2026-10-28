//! `DensityChart`: smoothed distributions as overlaid kernel density curves.
//! Curves come from `hydroxy_charts::density` (Gaussian kernel, Silverman's bandwidth).

use dioxus::prelude::*;
use hydroxy_charts::density::{self, Shape};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::probe::{Cell, Readout};

use crate::plot_kit::formatter;
use crate::plot_shell::{PlotShell, colour};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;

/// One density curve per group on shared axes, so their shapes compare directly.
/// Hover, touch or focus the plot and use the arrow keys to read every curve's density
/// at a value; Home and End jump, Escape clears. More than six groups fold into "Other".
#[component]
pub fn DensityChart(
    /// The groups, each with its raw values.
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Order value by channel". Used as the accessible name.
    label: String,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points. X values are data values, y values are densities.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let layout = density::density(&groups(), bandwidth, &annotations());
    let active = use_signal(|| None::<usize>);
    let fmt = formatter(format);
    let cur = if on_hover.is_some() || on_select.is_some() {
        active().and_then(|i| stop_datum(&layout.readout, i, 0, i, fmt))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    rsx! {
        PlotShell { scope: "density-chart", label, hint: "Use the left and right arrow keys to read the curves.", height, readout: layout.readout, format, active, datum: Some(datum), on_hover, on_select,
            {curves(&layout.shapes)}
        }
    }
}

/// The event datum for tooltip stop `i`: its heading as the label and its first row as the value.
pub(crate) fn stop_datum(
    readout: &Readout,
    i: usize,
    series: usize,
    index: usize,
    fmt: impl Fn(f64) -> String,
) -> Option<EventDatum> {
    let stop = readout.stops.get(i)?;
    let value = match stop.rows.first().map(|r| &r.value) {
        Some(Cell::Value(v) | Cell::Compact(v) | Cell::Small(v)) => *v,
        #[allow(clippy::cast_precision_loss, reason = "a count shown in a tooltip")]
        Some(Cell::Count(n)) => *n as f64,
        _ => stop.y,
    };
    Some(EventDatum { series, index, label: Some(stop.title.show(fmt)), value, node: None })
}

/// Filled curves with outlines, in paint order.
pub(crate) fn curves(shapes: &[Shape]) -> Element {
    rsx! {
        svg { "data-part": "marks", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
            for (i , s) in shapes.iter().enumerate() {
                g { key: "{i}", "data-part": "curve", style: "--i: {i};",
                    path { "data-part": "area", d: "{s.area}", fill: colour(s.colour) }
                    path { "data-part": "line", d: "{s.line}", stroke: colour(s.colour) }
                }
            }
        }
    }
}
