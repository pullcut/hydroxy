//! `<DensityChart>`: smoothed distributions as overlaid kernel density curves.
//! Curves come from `hydroxy_charts::density` (Gaussian kernel, Silverman's bandwidth).

use hydroxy_charts::density::{self, Shape};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::probe::{Cell, Readout};
use leptos::prelude::*;

use crate::plot_kit::{colour, formatter};
use crate::plot_shell::PlotShell;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;

/// One density curve per group on shared axes, so their shapes compare directly.
/// Hover, touch or focus the plot and use the arrow keys to read every curve's density
/// at a value; Home and End jump, Escape clears. More than six groups fold into "Other".
#[component]
pub fn DensityChart(
    /// The groups, each with its raw values.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Order value by channel". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    #[prop(optional, into)]
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points. X values are data values, y values are densities.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let layout = Memo::new(move |_| groups.with(|g| annotations.with(|a| density::density(g, bandwidth, a))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);
    let fmt = formatter(format);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            readout.with(|r| stop_datum(r, i, 0, i, fmt))
        })
    });

    view! {
        <PlotShell scope="density-chart" label hint="Use the left and right arrow keys to read the curves." height readout format active datum on_hover on_select>
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {move || curves(layout.with(|l| l.shapes.clone()))}
            </svg>
        </PlotShell>
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
        Some(Cell::Text(_)) | None => stop.y,
    };
    Some(EventDatum { series, index, label: Some(stop.title.show(fmt)), value, node: None })
}

/// Filled curves with outlines, in paint order.
pub(crate) fn curves(shapes: Vec<Shape>) -> impl IntoView + use<> {
    shapes
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let c = colour(s.colour);
            view! {
                <g data-part="curve" style=format!("--i: {i};")>
                    <path data-part="area" d=s.area fill=c.clone()></path>
                    <path data-part="line" d=s.line stroke=c></path>
                </g>
            }
        })
        .collect_view()
}
