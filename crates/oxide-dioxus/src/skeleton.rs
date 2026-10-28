//! `Skeleton`: a loading region that shows placeholder shapes until its
//! content is ready, then crossfades to it. `SkeletonText`, `SkeletonCircle`,
//! `SkeletonRect` and `SkeletonPreset` are the shapes, for building your own.
//!
//! The shimmer is one band sweeping across every shape at once; with reduced
//! motion the shapes hold still.

use dioxus::prelude::*;
use oxide_headless::skeleton::{self as h, Preset};

pub use oxide_headless::skeleton::Preset as SkeletonKind;

/// Ragged placeholder text: `lines` lines, the last one short.
#[component]
pub fn SkeletonText(
    /// Number of lines. Default 3.
    #[props(default = 3)]
    lines: usize,
    /// Changes the ragged widths; vary it between neighbouring blocks.
    #[props(default)]
    seed: u32,
) -> Element {
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "text",
            for w in h::line_widths(lines, seed) {
                span { "data-scope": h::SCOPE, "data-part": "line", style: h::line_style(w) }
            }
        }
    }
}

/// A round placeholder, for avatars.
#[component]
pub fn SkeletonCircle(
    /// Diameter in px. Default 40.
    #[props(default = 40)]
    size: u16,
) -> Element {
    rsx! { span { "data-scope": h::SCOPE, "data-part": "circle", style: h::circle_style(size) } }
}

/// A rectangular placeholder, for images and blocks. Any CSS lengths.
#[component]
pub fn SkeletonRect(
    /// Default `100%`.
    #[props(default = "100%".to_string())]
    width: String,
    /// Default `120px`.
    #[props(default = "120px".to_string())]
    height: String,
) -> Element {
    rsx! { span { "data-scope": h::SCOPE, "data-part": "rect", style: h::rect_style(&width, &height) } }
}

fn line(width: u8, size: Option<&'static str>) -> Element {
    rsx! { span { "data-scope": h::SCOPE, "data-part": "line", "data-size": size, style: h::line_style(width) } }
}

/// A placeholder shaped like an oxide component.
#[component]
pub fn SkeletonPreset(
    /// Which shape.
    kind: Preset,
    /// Changes the ragged widths.
    #[props(default)]
    seed: u32,
) -> Element {
    let body = match kind {
        Preset::Text(lines) => rsx! { SkeletonText { lines, seed } },
        Preset::Card => rsx! {
            SkeletonRect { height: "140px" }
            {line(h::line_widths(1, seed)[0], Some("heading"))}
            SkeletonText { lines: 2, seed: seed.wrapping_add(1) }
        },
        Preset::TableRows(rows, cols) => {
            let head = h::cell_widths(1, cols, seed.wrapping_add(7)).remove(0);
            let style = format!("grid-template-columns: repeat({cols}, minmax(0, 1fr));");
            rsx! {
                div { "data-scope": h::SCOPE, "data-part": "row", "data-head": "", style: style.clone(),
                    for w in head { {line(w / 2 + 20, None)} }
                }
                for row in h::cell_widths(rows, cols, seed) {
                    div { "data-scope": h::SCOPE, "data-part": "row", style: style.clone(),
                        for w in row { {line(w, None)} }
                    }
                }
            }
        }
        Preset::ListItem => rsx! {
            SkeletonCircle { size: 36 }
            SkeletonText { lines: 2, seed }
            SkeletonRect { width: "56px", height: "24px" }
        },
        Preset::Chart => rsx! {
            {line(40, Some("heading"))}
            div { "data-scope": h::SCOPE, "data-part": "plot",
                for v in h::chart_bars(12, seed) {
                    span { "data-scope": h::SCOPE, "data-part": "bar", style: "block-size: {v}%;" }
                }
            }
        },
        Preset::StatCard => rsx! {
            {line(45, None)}
            {line(60, Some("figure"))}
            {line(35, None)}
        },
    };
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "preset", "data-preset": kind.as_str(), {body} }
    }
}

/// A loading region. While `loading`, it is `aria-busy` and shows `fallback`
/// (or `preset`); when loading ends the content fades in over the placeholder.
#[component]
pub fn Skeleton(
    /// Whether the content is still loading.
    loading: ReadSignal<bool>,
    /// The placeholder shape when there's no `fallback`. Default three text lines.
    #[props(default)]
    preset: Preset,
    /// A custom placeholder built from the shape components.
    fallback: Option<Element>,
    /// Read to screen readers while loading. Default "Loading".
    #[props(default = "Loading".to_string())]
    label: String,
    /// The real content, rendered once loading ends.
    children: Element,
) -> Element {
    let busy = loading();
    let a = h::root_attrs(busy);
    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "aria-busy": a.aria_busy.to_string(),
            "data-state": a.data_state,
            span { "data-scope": h::SCOPE, "data-part": "status", role: "status",
                if busy { "{label}" }
            }
            div { "data-scope": h::SCOPE, "data-part": "placeholder", "aria-hidden": "true", "data-state": a.data_state,
                {fallback.unwrap_or_else(|| rsx! { SkeletonPreset { kind: preset } })}
            }
            if !busy {
                div { "data-scope": h::SCOPE, "data-part": "content", {children} }
            }
        }
    }
}
