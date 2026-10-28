//! `<BeeswarmChart>`: every value as a dot, packed so none overlap, optionally one row
//! per group. Geometry comes from `hydroxy_charts::swarm`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::HINT_STEP_VALUES;
use hydroxy_charts::swarm::{self, Arrange};
use leptos::prelude::*;

use crate::density_chart::stop_datum;
use crate::plot_kit::formatter;
use crate::plot_shell::PlotShell;
use crate::strip_plot_chart::dots;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;
pub use hydroxy_charts::scale::ScaleKind;

/// Dot diameter in px, matching the CSS.
const DOT: f64 = 8.0;
// packing assumes a 480px-wide plot, so narrow plots overlap a little; measure the plot if that matters.
const NOMINAL_WIDTH: f64 = 480.0;

/// Values stacked outwards from each row's centre line where they crowd, so the shape of
/// the distribution shows and every dot stays visible. Hover or touch to read a dot;
/// focus the plot and use the arrow keys to step through them, Home and End to jump,
/// Escape to clear.
#[component]
pub fn BeeswarmChart(
    /// The groups, one row each from the top. Pass one group for a single swarm.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Session length by plan". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Value axis mapping.
    #[prop(optional)]
    scale: ScaleKind,
    /// Reference lines, bands and labelled points. X values are data values, y values are row indices.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 260)]
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
    let arrange = Arrange::Swarm { dx: DOT / NOMINAL_WIDTH, dy: DOT / f64::from(height.max(1)) };
    let layout = Memo::new(move |_| groups.with(|g| annotations.with(|a| swarm::layout(g, scale, arrange, a))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);
    let fmt = formatter(format);
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let i = active.get()?;
            let (series, index) = layout.with(|l| {
                let group = l.dots.get(i)?.group;
                Some((group, l.dots[..i].iter().filter(|d| d.group == group).count()))
            })?;
            readout.with(|r| stop_datum(r, i, series, index, fmt))
        })
    });

    view! {
        <PlotShell scope="beeswarm-chart" label hint=HINT_STEP_VALUES height readout format active datum on_hover on_select>
            <div data-part="marks" aria-hidden="true">
                {move || dots(&layout.with(|l| l.dots.clone()), groups.with(Vec::len))}
            </div>
        </PlotShell>
    }
}
