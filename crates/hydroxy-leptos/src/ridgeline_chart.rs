//! `<RidgelineChart>`: one density curve per group, stacked in overlapping rows.
//! Curves come from `hydroxy_charts::density::ridgeline`.

use hydroxy_charts::boxplot::quantile;
use hydroxy_charts::density;
use hydroxy_charts::event::Datum as EventDatum;
use leptos::prelude::*;

use crate::density_chart::curves;
use crate::plot_shell::PlotShell;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;

/// Many distributions compared by shape, each ridge scaled to its own peak and allowed to
/// rise into the row above. Hover or touch a row to read it; focus the plot and use the
/// arrow keys to move between rows, Home and End to jump, Escape to clear.
#[component]
pub fn RidgelineChart(
    /// The groups, one ridge each from the top.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Commute time by month". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// How far a ridge may rise into the rows above, in rows.
    #[prop(default = 1.0)]
    overlap: f64,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    #[prop(optional, into)]
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points. X values are data values, y values are row indices.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 280)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
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
        Memo::new(move |_| groups.with(|g| annotations.with(|a| density::ridgeline(g, bandwidth, overlap, a))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);
    let datum = Signal::derive(move || {
        let g = active.get()?;
        groups.with(|gs| {
            let group = gs.get(g)?;
            let mut v: Vec<f64> = group.values.iter().copied().filter(|v| v.is_finite()).collect();
            v.sort_by(f64::total_cmp);
            let median = if v.is_empty() { 0.0 } else { quantile(&v, 0.5) };
            Some(EventDatum { series: g, index: g, label: Some(group.label.clone()), value: median, node: None })
        })
    });

    view! {
        <PlotShell scope="ridgeline-chart" label hint="Use the arrow keys to move between rows." height readout format active datum=Some(datum) on_hover on_select>
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {move || curves(layout.with(|l| l.shapes.clone()))}
            </svg>
        </PlotShell>
    }
}
