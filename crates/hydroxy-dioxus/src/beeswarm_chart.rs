//! `BeeswarmChart`: every value as a dot, packed so none overlap, optionally one row
//! per group. Geometry comes from `hydroxy_charts::swarm`.

use dioxus::prelude::*;
use hydroxy_charts::swarm::{self, Arrange};

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
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Session length by plan". Used as the accessible name.
    label: String,
    /// Value axis mapping.
    #[props(default)]
    scale: ScaleKind,
    /// Reference lines, bands and labelled points. X values are data values, y values are row indices.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 260)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let g = groups();
    let arrange = Arrange::Swarm { dx: DOT / NOMINAL_WIDTH, dy: DOT / f64::from(height.max(1)) };
    let layout = swarm::layout(&g, scale, arrange, &annotations());
    let active = use_signal(|| None::<usize>);
    rsx! {
        PlotShell { scope: "beeswarm-chart", label, hint: "Use the arrow keys to step through values.", height, readout: layout.readout, format, active,
            {dots(&layout.dots, g.len())}
        }
    }
}
