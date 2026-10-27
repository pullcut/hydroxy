//! `<RidgelineChart>`: one density curve per group, stacked in overlapping rows.
//! Curves come from `oxide_charts::density::ridgeline`.

use leptos::prelude::*;
use oxide_charts::density;

use crate::density_chart::curves;
use crate::plot_shell::PlotShell;

pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::boxplot::Group;

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
) -> impl IntoView {
    let layout = Memo::new(move |_| groups.with(|g| annotations.with(|a| density::ridgeline(g, bandwidth, overlap, a))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);

    view! {
        <PlotShell scope="ridgeline-chart" label hint="Use the arrow keys to move between rows." height readout format active>
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {move || curves(layout.with(|l| l.shapes.clone()))}
            </svg>
        </PlotShell>
    }
}
