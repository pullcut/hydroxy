//! Shared parts for the set and part-of-whole charts (`UpSet`, Venn, Voronoi treemap,
//! nested donut and alluvial): the anchored tooltip.

use hydroxy_charts::plot::Tip;
use leptos::prelude::*;

use crate::plot_kit::ChartTip;

/// The tooltip for `tip`, pinned at its anchor in % of the plot.
#[allow(unreachable_pub, reason = "#[component] makes the generated props struct pub")]
#[component]
pub(crate) fn SetsTip(tip: Tip) -> impl IntoView {
    view! { <ChartTip tip /> }
}
