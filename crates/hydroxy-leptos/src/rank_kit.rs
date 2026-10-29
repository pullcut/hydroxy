//! Shared parts for the series charts that dim the others (bump, radar, slope), the group
//! legend of the node charts (arc diagram, network), and the index stepping shared by the
//! band charts (forecast fan, range area).

use hydroxy_charts::line::{Series, nearest_index};
use hydroxy_charts::nav::list_key;
use hydroxy_charts::plot::{self, Tick, colour};
use leptos::prelude::*;

/// Which series the viewer is looking at: the active mark's, else the hovered legend item.
/// Every other series is dimmed.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Dim {
    /// The active `(series, point)`.
    pub active: RwSignal<Option<(usize, usize)>>,
    /// The legend item under the pointer.
    pub hover: RwSignal<Option<usize>>,
}

impl Dim {
    /// A tracker reading the chart's active mark.
    pub(crate) fn new(active: RwSignal<Option<(usize, usize)>>) -> Self {
        Self { active, hover: RwSignal::new(None) }
    }

    /// The `data-dim` value for series `i`.
    pub(crate) fn of(self, i: usize) -> Option<&'static str> {
        let focus = self.active.get().map(|(s, _)| s).or_else(|| self.hover.get());
        focus.is_some_and(|f| f != i).then_some("")
    }
}

/// The `--c` style that gives series `i` its colour.
pub(crate) fn series_style(i: usize) -> String {
    format!("--c: {};", colour(i))
}

/// The series legend. Hovering an item dims the other series. Hidden for one series.
#[allow(unreachable_pub, reason = "#[component] makes the generated props struct pub")]
#[component]
pub(crate) fn SeriesLegend(
    /// The series, in colour order.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// The chart's dimming state.
    dim: Dim,
) -> impl IntoView {
    view! {
        <Show when=move || series.with(|s| s.len() > 1)>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    series
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, s)| {
                            view! {
                                <li
                                    data-part="legend-item"
                                    data-index=i
                                    data-dim=move || dim.of(i)
                                    on:pointerenter=move |_| dim.hover.set(Some(i))
                                    on:pointerleave=move |_| dim.hover.set(None)
                                >
                                    <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                    {s.name}
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
        </Show>
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
#[allow(unreachable_pub, reason = "#[component] makes the generated props struct pub")]
#[component]
pub(crate) fn GroupLegend(
    /// The distinct group ids.
    #[prop(into)]
    groups: Signal<Vec<usize>>,
    /// A group's name.
    #[prop(into)]
    name: Callback<usize, String>,
    /// A group's CSS colour.
    colour: fn(usize) -> String,
) -> impl IntoView {
    view! {
        <Show when=move || groups.with(|g| g.len() > 1)>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    groups
                        .get()
                        .into_iter()
                        .map(|g| {
                            view! {
                                <li data-part="legend-item" data-group=g>
                                    <span data-part="swatch" style=format!("background: {};", colour(g))></span>
                                    {name.run(g)}
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
        </Show>
    }
}

/// Pointer position on the plot, or `None` when it leaves.
type PointHandler = Callback<Option<(f64, f64)>>;
/// Key name; returns whether the key was handled.
type KeyHandler = Callback<String, bool>;

/// Pointer and key handlers for a chart that steps through `count` points at positions
/// `xs` (0..=1): `(on_point, on_key)` for [`crate::plot_kit::Plot`].
pub(crate) fn index_handlers(
    active: RwSignal<Option<usize>>,
    xs: Signal<Vec<f64>>,
    count: Signal<usize>,
) -> (PointHandler, KeyHandler) {
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, _)| xs.with(|xs| nearest_index(xs, x)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        list_key(&key, active.get_untracked(), count.get_untracked()).map(|next| active.set(next)).is_some()
    });
    (on_point, on_key)
}

/// The x-axis labels for points named `names` at positions `xs`, at most eight.
pub(crate) fn index_labels(names: Signal<Vec<String>>, xs: Signal<Vec<f64>>) -> Signal<Vec<Tick>> {
    Signal::derive(move || names.with(|n| xs.with(|xs| plot::point_ticks(n, xs, 8))))
}
