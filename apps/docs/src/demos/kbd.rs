use hydroxy_leptos::kbd::Kbd;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="display: grid; gap: var(--space-4); font: var(--text-sm); color: var(--text-2);">
            <p style="margin: 0; display: flex; align-items: center; gap: var(--space-2);">
                "Search everything" <Kbd>"⌘"</Kbd> <Kbd>"K"</Kbd>
            </p>
            <p style="margin: 0; display: flex; align-items: center; gap: var(--space-2);">
                "New line" <Kbd>"Shift"</Kbd> <Kbd>"Enter"</Kbd>
            </p>
            <p style="margin: 0; display: flex; align-items: center; gap: var(--space-2);">
                "Close the panel" <Kbd>"Esc"</Kbd>
            </p>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "kbd",
    name: "Kbd",
    group: "Display",
    summary: "A keyboard shortcut hint. One key per Kbd; put several side by side for a combination.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("kbd.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/kbd.rs"),
    parts: &[&["Kbd", "(root)", "<kbd data-scope=kbd>"]],
    keys: &[],
    props: &[&["children", "View", "The key, such as ⌘ or K"]],
    events: &[],
};
