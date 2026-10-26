use leptos::prelude::*;
use oxide_leptos::stat_card::{DeltaTone, Jewel, StatCard};

pub fn demo() -> AnyView {
    view! {
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: var(--space-5);">
            <StatCard
                label="Interviews read"
                value="22 of 24"
                delta="+6"
                delta_tone=DeltaTone::Up
                caption="this week"
                jewel=Jewel::Amethyst
            />
            <StatCard
                label="Median reading time"
                value="11"
                unit="min"
                delta="−4 min"
                delta_tone=DeltaTone::Up
                caption="vs last week"
                jewel=Jewel::Sapphire
            />
            <StatCard label="Open flags" value="2" delta="+1" delta_tone=DeltaTone::Down caption="since Monday" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "stat-card",
    name: "Stat card",
    group: "Data",
    summary: "One key figure with its name and change. Lead with the number; the diamond ties a card to its chart series.",
    demo,
    leptos: include_str!("stat_card.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/stat_card.rs"),
    parts: &[
        &["StatCard", "—", "div[data-scope=stat-card]"],
        &["", "label", "What the figure measures, with an optional jewel diamond"],
        &["", "value", "The figure in mono, with an optional unit"],
        &["", "delta", "The change, coloured by delta_tone"],
    ],
    keys: &[],
    props: &[
        &["label", "String", "Required. Sentence case"],
        &["value", "String", "Required. The figure"],
        &["unit", "String", "After the figure, such as min"],
        &["delta", "String", "The change, such as −4 min"],
        &["delta_tone", "DeltaTone", "Up (good), Down (bad) or Neutral (default)"],
        &["jewel", "Jewel", "Diamond beside the label"],
        &["caption", "String", "Context after the change"],
    ],
};
