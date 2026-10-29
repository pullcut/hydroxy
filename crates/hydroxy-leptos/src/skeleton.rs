//! `<Skeleton>`: a loading region that shows placeholder shapes until its
//! content is ready, then crossfades to it. `<SkeletonText>`, `<SkeletonCircle>`,
//! `<SkeletonRect>` and `<SkeletonPreset>` are the shapes, for building your own.
//!
//! The shimmer is one band sweeping across every shape at once; with reduced
//! motion the shapes hold still.

use hydroxy_headless::skeleton::{self as h, Preset};
use leptos::prelude::*;

pub use hydroxy_headless::skeleton::Preset as SkeletonKind;

/// Ragged placeholder text: `lines` lines, the last one short.
#[component]
pub fn SkeletonText(
    /// Number of lines. Default 3.
    #[prop(default = 3)]
    lines: usize,
    /// Changes the ragged widths; vary it between neighbouring blocks.
    #[prop(optional)]
    seed: u32,
) -> impl IntoView {
    view! {
        <div data-scope=h::SCOPE data-part="text">
            {h::line_widths(lines, seed)
                .into_iter()
                .map(|w| view! { <span data-scope=h::SCOPE data-part="line" style=h::line_style(w)></span> })
                .collect_view()}
        </div>
    }
}

/// A round placeholder, for avatars.
#[component]
pub fn SkeletonCircle(
    /// Diameter in px. Default 40.
    #[prop(default = 40)]
    size: u16,
) -> impl IntoView {
    view! { <span data-scope=h::SCOPE data-part="circle" style=h::circle_style(size)></span> }
}

/// A rectangular placeholder, for images and blocks. Any CSS lengths.
#[component]
pub fn SkeletonRect(
    /// Default `100%`.
    #[prop(default = "100%".into(), into)]
    width: String,
    /// Default `120px`.
    #[prop(default = "120px".into(), into)]
    height: String,
) -> impl IntoView {
    view! { <span data-scope=h::SCOPE data-part="rect" style=h::rect_style(&width, &height)></span> }
}

/// A placeholder shaped like an hydroxy component.
#[component]
pub fn SkeletonPreset(
    /// Which shape.
    kind: Preset,
    /// Changes the ragged widths.
    #[prop(optional)]
    seed: u32,
) -> impl IntoView {
    let body = match kind {
        Preset::Text(lines) => view! { <SkeletonText lines=lines seed=seed /> }.into_any(),
        Preset::Card => view! {
            <SkeletonRect height="140px" />
            <span data-scope=h::SCOPE data-part="line" data-size="heading" style=h::line_style(h::line_widths(1, seed)[0])></span>
            <SkeletonText lines=2 seed=seed.wrapping_add(1) />
        }
        .into_any(),
        Preset::TableRows(rows, cols) => {
            let head = h::cell_widths(1, cols, seed.wrapping_add(7)).remove(0);
            let style = format!("grid-template-columns: repeat({cols}, minmax(0, 1fr));");
            view! {
                <div data-scope=h::SCOPE data-part="row" data-head="" style=style.clone()>
                    {head.into_iter().map(|w| view! { <span data-scope=h::SCOPE data-part="line" style=h::line_style(w / 2 + 20)></span> }).collect_view()}
                </div>
                {h::cell_widths(rows, cols, seed)
                    .into_iter()
                    .map(|row| view! {
                        <div data-scope=h::SCOPE data-part="row" style=style.clone()>
                            {row.into_iter().map(|w| view! { <span data-scope=h::SCOPE data-part="line" style=h::line_style(w)></span> }).collect_view()}
                        </div>
                    })
                    .collect_view()}
            }
            .into_any()
        }
        Preset::ListItem => view! {
            <SkeletonCircle size=36 />
            <SkeletonText lines=2 seed=seed />
            <SkeletonRect width="56px" height="24px" />
        }
        .into_any(),
        Preset::Chart => view! {
            <span data-scope=h::SCOPE data-part="line" data-size="heading" style=h::line_style(40)></span>
            <div data-scope=h::SCOPE data-part="plot">
                {h::chart_bars(12, seed)
                    .into_iter()
                    .map(|v| view! { <span data-scope=h::SCOPE data-part="bar" style=format!("block-size: {v}%;")></span> })
                    .collect_view()}
            </div>
        }
        .into_any(),
        Preset::StatCard => view! {
            <span data-scope=h::SCOPE data-part="line" style=h::line_style(45)></span>
            <span data-scope=h::SCOPE data-part="line" data-size="figure" style=h::line_style(60)></span>
            <span data-scope=h::SCOPE data-part="line" style=h::line_style(35)></span>
        }
        .into_any(),
    };
    view! { <div data-scope=h::SCOPE data-part="preset" data-preset=kind.as_str()>{body}</div> }
}

/// A loading region. While `loading`, it is `aria-busy` and shows `fallback`
/// (or `preset`); when loading ends the content fades in over the placeholder.
#[component]
pub fn Skeleton(
    /// Whether the content is still loading.
    #[prop(into)]
    loading: Signal<bool>,
    /// The placeholder shape when there's no `fallback`. Default three text lines.
    #[prop(optional)]
    preset: Preset,
    /// A custom placeholder built from the shape components.
    #[prop(optional, into)]
    fallback: Option<ViewFn>,
    /// Read to screen readers while loading. Default "Loading".
    #[prop(default = "Loading".into(), into)]
    label: String,
    /// The real content, rendered once loading ends.
    children: ChildrenFn,
) -> impl IntoView {
    let attrs = move || h::root_attrs(loading.get());
    let fallback = StoredValue::new(fallback);
    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            aria-busy=move || attrs().aria_busy.to_string()
            data-state=move || attrs().data_state
        >
            <span data-scope=h::SCOPE data-part="status" role="status">
                {move || loading.get().then(|| label.clone())}
            </span>
            <div data-scope=h::SCOPE data-part="placeholder" aria-hidden="true" data-state=move || attrs().data_state>
                {move || match fallback.get_value() {
                    Some(f) => f.run(),
                    None => view! { <SkeletonPreset kind=preset /> }.into_any(),
                }}
            </div>
            <Show when=move || !loading.get()>
                <div data-scope=h::SCOPE data-part="content">{children()}</div>
            </Show>
        </div>
    }
}
