//! Shared parts for the zoomable hierarchy charts (sunburst, treemap, bubble pack, icicle
//! and flame graph, tree): the breadcrumb trail, keyboard handling and legends.
#![allow(dead_code, reason = "adopted by the hierarchy charts one by one")]

use dioxus::prelude::*;
use hydroxy_charts::hierarchy::{Drill, drill_key};
use hydroxy_charts::nav;
use hydroxy_charts::plot::colour;

/// One breadcrumb: the node's name and its path from the root.
pub(crate) type Crumb = (String, Vec<usize>);

/// The zoom trail. Every crumb but the last is a button that calls `go` with its path.
#[component]
pub(crate) fn Breadcrumb(
    /// The trail from the root to the zoomed node, never empty.
    crumbs: ReadSignal<Vec<Crumb>>,
    /// Zoom to a path.
    go: Callback<Vec<usize>>,
) -> Element {
    let crumbs = crumbs();
    let last = crumbs.len().saturating_sub(1);
    rsx! {
        nav { "data-part": "breadcrumb", "aria-label": "Zoom level",
            ol {
                for (i , (name , path)) in crumbs.into_iter().enumerate() {
                    li {
                        if i == last {
                            span { "data-part": "crumb", "aria-current": "location", "{name}" }
                        } else {
                            button {
                                r#type: "button",
                                "data-part": "crumb",
                                onclick: move |_| go.call(path.clone()),
                                "{name}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Handle a key on a chart that zooms with [`drill_key`] (sunburst, treemap): move the
/// active mark, zoom into one with `into`, or out with `up`. Returns whether the key was
/// handled, so the caller can `prevent_default`.
pub(crate) fn drill(
    key: &str,
    mut active: Signal<Option<usize>>,
    zoomable: &[bool],
    at_top: bool,
    into: impl FnOnce(usize),
    up: impl FnOnce(),
) -> bool {
    let current = *active.peek();
    match drill_key(key, current, zoomable, at_top) {
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
    items: ReadSignal<Vec<String>>,
) -> Element {
    let items = items();
    if items.len() <= 1 {
        return rsx! {};
    }
    rsx! {
        ul { "data-part": "legend", "aria-hidden": "true",
            for (i , name) in items.into_iter().enumerate() {
                li { "data-part": "legend-item",
                    span { "data-part": "swatch", style: "background: {colour(i)};" }
                    "{name}"
                }
            }
        }
    }
}

/// A legend that lights the mark you hover: one entry per `(mark index, name, colour slot)`.
/// Hidden for one entry.
#[component]
pub(crate) fn MarkLegend(
    /// The entries.
    items: ReadSignal<Vec<(usize, String, usize)>>,
    /// The active mark, shared with the plot.
    active: Signal<Option<usize>>,
) -> Element {
    let items = items();
    if items.len() <= 1 {
        return rsx! {};
    }
    let mut active = active;
    rsx! {
        ul { "data-part": "legend", "aria-hidden": "true",
            for (i , name , slot) in items {
                li {
                    "data-part": "legend-item",
                    "data-active": (active() == Some(i)).then_some(""),
                    onpointerenter: move |_| {
                        if *active.peek() != Some(i) {
                            active.set(Some(i));
                        }
                    },
                    onpointerleave: move |_| active.set(None),
                    span { "data-part": "swatch", style: "background: {colour(slot)};" }
                    "{name}"
                }
            }
        }
    }
}
