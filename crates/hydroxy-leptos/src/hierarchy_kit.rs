//! Shared parts for the zoomable hierarchy charts (sunburst, treemap, bubble pack, icicle
//! and flame graph, tree): the breadcrumb trail, keyboard handling and legends.

use hydroxy_charts::hierarchy::{Drill, drill_key};
use hydroxy_charts::nav;
use hydroxy_charts::plot::colour;
use leptos::either::Either;
use leptos::prelude::*;

/// One breadcrumb: the node's name and its path from the root.
pub(crate) type Crumb = (String, Vec<usize>);

/// The zoom trail. Every crumb but the last is a button that calls `go` with its path.
#[component]
pub(crate) fn Breadcrumb(
    /// The trail from the root to the zoomed node, never empty.
    #[prop(into)]
    crumbs: Signal<Vec<Crumb>>,
    /// Zoom to a path.
    go: Callback<Vec<usize>>,
) -> impl IntoView {
    view! {
        <nav data-part="breadcrumb" aria-label="Zoom level">
            <ol>
                {move || {
                    let c = crumbs.get();
                    let last = c.len().saturating_sub(1);
                    c.into_iter()
                        .enumerate()
                        .map(|(i, (name, path))| {
                            if i == last {
                                Either::Left(view! { <li><span data-part="crumb" aria-current="location">{name}</span></li> })
                            } else {
                                Either::Right(view! {
                                    <li>
                                        <button type="button" data-part="crumb" on:click=move |_| go.run(path.clone())>{name}</button>
                                    </li>
                                })
                            }
                        })
                        .collect_view()
                }}
            </ol>
        </nav>
    }
}

/// Handle a key on a chart that zooms with [`drill_key`] (sunburst, treemap): move the
/// active mark, zoom into one with `into`, or out with `up`. Returns whether the key was
/// handled, so the caller can `prevent_default`.
pub(crate) fn drill(
    key: &str,
    active: RwSignal<Option<usize>>,
    zoomable: &[bool],
    at_top: bool,
    into: impl FnOnce(usize),
    up: impl FnOnce(),
) -> bool {
    match drill_key(key, active.get_untracked(), zoomable, at_top) {
        Some(Drill::Move(next)) => active.set(next),
        Some(Drill::Into(i)) => into(i),
        Some(Drill::Up) => up(),
        None => return false,
    }
    true
}

/// What a key does on a chart whose marks form one row (icicle, flame graph, bubble pack).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZoomKey {
    /// Enter or Space on mark `i`: zoom in, or out if it is the zoomed node.
    Activate(usize),
    /// Escape with nothing active while zoomed in: zoom out one level.
    Up,
    /// Move the active mark (or clear it with `None`).
    Move(Option<usize>),
}

/// The key's meaning for `len` marks with `current` active. `zoomed` is true below the root.
/// Down and Up act as Right and Left.
pub(crate) fn zoom_key(key: &str, current: Option<usize>, len: usize, zoomed: bool) -> Option<ZoomKey> {
    if key == "Enter" || key == " " {
        return current.map(ZoomKey::Activate);
    }
    if key == "Escape" && current.is_none() && zoomed {
        return Some(ZoomKey::Up);
    }
    nav::list_key(nav::linear_key(key), current, len).map(ZoomKey::Move)
}

/// A legend of the branch names under the zoomed node, coloured by position. Hidden for one entry.
#[component]
pub(crate) fn PartitionLegend(
    /// Names from `partition::legend`.
    #[prop(into)]
    items: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || items.with(|l| l.len() > 1)>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    items
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, name)| view! { <li data-part="legend-item"><span data-part="swatch" style=format!("background: {};", colour(i))></span>{name}</li> })
                        .collect_view()
                }}
            </ul>
        </Show>
    }
}

/// A legend that lights the mark you hover: one entry per `(mark index, name, colour slot)`.
/// Hidden for one entry.
#[component]
pub(crate) fn MarkLegend(
    /// The entries.
    #[prop(into)]
    items: Signal<Vec<(usize, String, usize)>>,
    /// The active mark, shared with the plot.
    active: RwSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <Show when=move || items.with(|l| l.len() > 1)>
            <ul data-part="legend" aria-hidden="true">
                {move || {
                    items
                        .get()
                        .into_iter()
                        .map(|(i, name, slot)| {
                            view! {
                                <li
                                    data-part="legend-item"
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    on:pointerenter=move |_| {
                                        if active.get_untracked() != Some(i) {
                                            active.set(Some(i));
                                        }
                                    }
                                    on:pointerleave=move |_| active.set(None)
                                >
                                    <span data-part="swatch" style=format!("background: {};", colour(slot))></span>
                                    {name}
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
        </Show>
    }
}
