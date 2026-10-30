use hydroxy_leptos::button::{Button, ButtonVariant};
use hydroxy_leptos::skeleton::{Skeleton, SkeletonCircle, SkeletonKind, SkeletonText};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (loading, set_loading) = signal(true);

    view! {
        <div style="display: grid; gap: var(--space-6);">
            <div>
                <Button variant=ButtonVariant::Secondary on_click=move |_| set_loading.update(|l| *l = !*l)>
                    {move || if loading.get() { "Show content" } else { "Reload" }}
                </Button>
            </div>

            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: var(--space-4);">
                // Presets are sized like the components they stand in for.
                <Skeleton loading=loading preset=SkeletonKind::StatCard label="Loading revenue">
                    <div data-scope="stat-card">
                        <span style="font: var(--text-sm); color: var(--text-2);">"Monthly revenue"</span>
                        <span style="font: var(--text-figure); font-size: 30px;">"$48,210"</span>
                        <span style="font: var(--text-xs); color: var(--text-3);">"+12.4% on September"</span>
                    </div>
                </Skeleton>
                <Skeleton loading=loading preset=SkeletonKind::StatCard label="Loading signups">
                    <div data-scope="stat-card">
                        <span style="font: var(--text-sm); color: var(--text-2);">"New signups"</span>
                        <span style="font: var(--text-figure); font-size: 30px;">"1,284"</span>
                        <span style="font: var(--text-xs); color: var(--text-3);">"+3.1% on September"</span>
                    </div>
                </Skeleton>
            </div>

            // A custom placeholder built from the shapes.
            <Skeleton
                loading=loading
                label="Loading comment"
                fallback=|| view! {
                    <div style="display: flex; gap: var(--space-3);">
                        <SkeletonCircle size=36 />
                        <div style="flex: 1;"><SkeletonText lines=3 seed=4 /></div>
                    </div>
                }
            >
                <p style="margin: 0; font: var(--text-body);">
                    <strong>"Priya Natarajan"</strong>
                    " The churn dip lines up with the onboarding change on the 14th. Worth a cohort split before we call it."
                </p>
            </Skeleton>

            <Skeleton loading=loading preset=SkeletonKind::TableRows(4, 4) label="Loading invoices">
                <p style="margin: 0;">"4 invoices, all paid."</p>
            </Skeleton>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "skeleton",
    name: "Skeleton",
    group: "Feedback",
    summary: "Placeholder shapes that hold the layout while content loads, then fade into the real thing. One soft shimmer sweeps across them all; with reduced motion they stay still.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("skeleton.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/skeleton.rs"),
    parts: &[
        &["Skeleton", "root", "The loading region. aria-busy while loading, data-state loading or ready"],
        &["Skeleton", "status", "Visually hidden role=\"status\" text, read while loading"],
        &["Skeleton", "placeholder", "The shapes, aria-hidden. Fades out when ready"],
        &["Skeleton", "content", "The real content, rendered once loading ends. Fades in"],
        &["SkeletonText", "text / line", "Ragged lines; the last one is short"],
        &["SkeletonCircle", "circle", "Round block, for avatars"],
        &["SkeletonRect", "rect", "Rectangle, for media and blocks"],
        &["SkeletonPreset", "preset", "Card, table rows, list item, chart or stat card (data-preset)"],
    ],
    keys: &[],
    props: &[
        &["loading", "bool", "Required. Shows the placeholder while true"],
        &["preset", "SkeletonKind", "Placeholder shape when there's no fallback. Default three text lines"],
        &["fallback", "ViewFn", "A custom placeholder built from the shapes"],
        &["label", "String", "Read to screen readers while loading. Default \"Loading\""],
        &["lines / seed", "usize / u32", "SkeletonText: line count, and which ragged widths"],
    ],
    events: &[],
};
