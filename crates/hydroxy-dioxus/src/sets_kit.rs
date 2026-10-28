//! Shared parts for the set and part-of-whole charts (`UpSet`, Venn, Voronoi treemap,
//! nested donut and alluvial): the anchored tooltip.

use dioxus::prelude::*;
use hydroxy_charts::plot::Tip;

use crate::plot_kit::ChartTip;

/// The tooltip for `tip`, pinned at its anchor in % of the plot.
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub(crate) fn SetsTip(tip: Tip) -> Element {
    rsx! {
        ChartTip { tip }
    }
}
