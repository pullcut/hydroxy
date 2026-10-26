use leptos::prelude::*;
use oxide_leptos::reveal::{Reveal, RevealReason};

fn card(title: &'static str, body: &'static str) -> impl IntoView {
    view! {
        <div style="display: grid; gap: var(--space-2); padding: var(--space-6); border: 1px solid var(--border-1); border-radius: var(--radius-md); background: var(--surface-card);">
            <strong style="font: var(--text-ui);">{title}</strong>
            <span style="font: var(--text-sm); color: var(--text-3);">{body}</span>
        </div>
    }
}

pub fn demo() -> AnyView {
    let (why, set_why) = signal(String::from("Scroll down. Each card fades in as it arrives."));

    view! {
        <div style="display: grid; gap: 38vh;">
            <p style="margin: 0; font: var(--text-label); color: var(--text-3);">{why}</p>
            <Reveal on_reveal=move |r: RevealReason| set_why.set(format!("Last revealed: {r:?}."))>
                {card("Import", "Bring in notes, tickets and survey exports.")}
            </Reveal>
            <Reveal on_reveal=move |r: RevealReason| set_why.set(format!("Last revealed: {r:?}."))>
                {card("Tag", "Group what you find into themes.")}
            </Reveal>
            <Reveal on_reveal=move |r: RevealReason| set_why.set(format!("Last revealed: {r:?}."))>
                {card("Share", "Send a digest to the team.")}
            </Reveal>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "reveal",
    name: "Reveal",
    group: "Display",
    summary: "Content that fades in once it scrolls into view. One-way: it stays shown. Visible with no script, and shown at once for people who prefer reduced motion.",
    demo,
    leptos: include_str!("reveal.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/reveal.rs"),
    parts: &[
        &["Reveal", "root", "Wrapper <div>. data-state=hidden|shown. data-armed once the browser is watching it; the hidden look only applies then"],
    ],
    keys: &[],
    props: &[
        &["children", "Children", "The content to reveal"],
        &["default_shown", "bool", "Start shown, with no animation"],
        &["on_reveal", "Callback<RevealReason>", "Called once: Immediate, Intersection, Backstop or Manual"],
    ],
};
