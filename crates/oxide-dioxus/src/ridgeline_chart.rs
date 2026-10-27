//! `RidgelineChart`: one density curve per group, stacked in overlapping rows.
//! Curves come from `oxide_charts::density::ridgeline`.

use dioxus::prelude::*;
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
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Commute time by month". Used as the accessible name.
    label: String,
    /// How far a ridge may rise into the rows above, in rows.
    #[props(default = 1.0)]
    overlap: f64,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points. X values are data values, y values are row indices.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 280)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let layout = density::ridgeline(&groups(), bandwidth, overlap, &annotations());
    let active = use_signal(|| None::<usize>);
    rsx! {
        PlotShell { scope: "ridgeline-chart", label, hint: "Use the arrow keys to move between rows.", height, readout: layout.readout, format, active,
            {curves(&layout.shapes)}
        }
    }
}
