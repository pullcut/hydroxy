//! `DensityChart`: smoothed distributions as overlaid kernel density curves.
//! Curves come from `oxide_charts::density` (Gaussian kernel, Silverman's bandwidth).

use dioxus::prelude::*;
use oxide_charts::density::{self, Shape};

use crate::plot_shell::{PlotShell, colour};

pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::boxplot::Group;

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
) -> Element {
    let layout = density::density(&groups(), bandwidth, &annotations());
    let active = use_signal(|| None::<usize>);
    rsx! {
        PlotShell { scope: "density-chart", label, hint: "Use the left and right arrow keys to read the curves.", height, readout: layout.readout, format, active,
            {curves(&layout.shapes)}
        }
    }
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
