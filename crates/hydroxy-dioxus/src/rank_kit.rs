//! Shared parts for the series charts that dim the others (bump, radar, slope), the group
//! legend of the node charts (arc diagram, network), and the index stepping shared by the
//! band charts (forecast fan, range area).
#![allow(dead_code, reason = "adopted by the charts one by one")]

use dioxus::prelude::*;
use hydroxy_charts::line::{Series, nearest_index};
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tick, colour};

/// Which series the viewer is looking at: the active mark's, else the hovered legend item.
/// Every other series is dimmed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Dim {
    /// The active `(series, point)`.
    pub active: Signal<Option<(usize, usize)>>,
    /// The legend item under the pointer.
    pub hover: Signal<Option<usize>>,
}

impl Dim {
    /// A tracker reading the chart's active mark. Call it once per render, like a hook.
    pub(crate) fn new(active: Signal<Option<(usize, usize)>>) -> Self {
        Self { active, hover: use_signal(|| None) }
    }

    /// The `data-dim` value for series `i`.
    pub(crate) fn of(self, i: usize) -> Option<&'static str> {
        let focus = (self.active)().map(|(s, _)| s).or_else(|| (self.hover)());
        focus.is_some_and(|f| f != i).then_some("")
    }
}

/// The `--c` style that gives series `i` its colour.
pub(crate) fn series_style(i: usize) -> String {
    format!("--c: {};", colour(i))
}

/// The series legend. Hovering an item dims the other series. Hidden for one series.
#[component]
pub(crate) fn SeriesLegend(
    /// The series, in colour order.
    series: ReadSignal<Vec<Series>>,
    /// The chart's dimming state.
    dim: Dim,
) -> Element {
    let series = series();
    if series.len() <= 1 {
        return rsx! {};
    }
    let mut hover = dim.hover;
    rsx! {
        ul { "data-part": "legend", "aria-hidden": "true",
            for (i , s) in series.into_iter().enumerate() {
                li {
                    "data-part": "legend-item",
                    "data-index": "{i}",
                    "data-dim": dim.of(i),
                    onpointerenter: move |_| hover.set(Some(i)),
                    onpointerleave: move |_| hover.set(None),
                    span { "data-part": "swatch", style: "background: {colour(i)};" }
                    "{s.name}"
                }
            }
        }
    }
}

/// The name of node group `group`: from `names`, else "Group N".
pub(crate) fn group_name(names: &[String], group: usize) -> String {
    names.get(group).cloned().unwrap_or_else(|| format!("Group {}", group + 1))
}

/// The distinct values of `groups`, ascending.
pub(crate) fn distinct(groups: impl IntoIterator<Item = usize>) -> Vec<usize> {
    let mut g: Vec<usize> = groups.into_iter().collect();
    g.sort_unstable();
    g.dedup();
    g
}

/// The legend of node groups: one swatch per group id. Hidden for one group.
#[component]
pub(crate) fn GroupLegend(
    /// The distinct group ids.
    groups: ReadSignal<Vec<usize>>,
    /// A group's name.
    name: Callback<usize, String>,
    /// A group's CSS colour, such as `Callback::new(plot::colour)`.
    colour: Callback<usize, String>,
) -> Element {
    let groups = groups();
    if groups.len() <= 1 {
        return rsx! {};
    }
    rsx! {
        ul { "data-part": "legend", "aria-hidden": "true",
            for g in groups {
                li { "data-part": "legend-item", "data-group": "{g}",
                    span { "data-part": "swatch", style: "background: {colour.call(g)};" }
                    "{name.call(g)}"
                }
            }
        }
    }
}

/// Pointer position on the plot, or `None` when it leaves.
type PointHandler = Callback<Option<(f64, f64)>>;
/// Key name; returns whether the key was handled.
type KeyHandler = Callback<String, bool>;

/// Pointer and key handlers for a chart that steps through `count` points at positions
/// `xs` (0..=1): `(on_point, on_key)` for [`crate::plot_kit::Plot`].
pub(crate) fn index_handlers(
    mut active: Signal<Option<usize>>,
    xs: ReadSignal<Vec<f64>>,
    count: ReadSignal<usize>,
) -> (PointHandler, KeyHandler) {
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| nearest_index(&xs.peek(), x));
        if *active.peek() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        let current = *active.peek();
        list_key(&key, current, *count.peek()).map(|next| active.set(next)).is_some()
    });
    (on_point, on_key)
}

/// The x-axis labels for points named `names` at positions `xs`, at most eight.
pub(crate) fn index_labels(names: &[String], xs: &[f64]) -> Vec<Tick> {
    plot::point_ticks(names, xs, 8)
}
