//! `FlameGraphChart`: a profile as stacked frames, callers below callees, each frame as
//! wide as its total time. Shares the partition layout and markup with
//! [`IcicleChart`](crate::icicle_chart::IcicleChart).

use dioxus::prelude::*;

use crate::icicle_chart::PartitionChart;
pub use oxide_charts::hierarchy::Tree;

/// A flame graph. A branch's `value` is its total time when that exceeds its children's,
/// and the rest is self time. Frames colour by a hash of their name, so a function keeps
/// its colour. Hover, touch or focus the chart and use the arrow keys to read frames;
/// click one, or press Enter, to zoom in. Escape, the bottom row or the breadcrumb climb out.
#[component]
pub fn FlameGraphChart(
    /// The call tree, root at the bottom.
    root: ReadSignal<Tree>,
    /// What the chart shows, such as "CPU time in render". Used as the accessible name.
    label: String,
    /// Frames whose name contains this, ignoring case, are marked and the rest dim.
    #[props(default)]
    highlight: ReadSignal<String>,
    /// Plot height in px at an 800px drawing width.
    #[props(default = 320)]
    height: u16,
    /// Formats times for the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    rsx! {
        PartitionChart { root, label, height, format, highlight: Some(highlight) }
    }
}
