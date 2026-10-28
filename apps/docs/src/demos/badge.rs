use hydroxy_leptos::badge::{Badge, Tone};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
            <Badge tone=Tone::Success dot=true>"Ready"</Badge>
            <Badge tone=Tone::Warning>"2 flags"</Badge>
            <Badge tone=Tone::Danger>"Failed"</Badge>
            <Badge tone=Tone::Info dot=true>"In review"</Badge>
            <Badge tone=Tone::Accent>"New"</Badge>
            <Badge>"Draft"</Badge>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "badge",
    name: "Badge",
    group: "Display",
    summary: "A small status label for states like Ready or 2 flags. Each tone pairs a soft background with a strong text colour.",
    demo,
    leptos: include_str!("badge.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/badge.rs"),
    parts: &[
        &["Badge", "(root)", "<span data-scope=badge data-tone>"],
        &["(inside)", "dot", "Optional 6px dot in the text colour, aria-hidden"],
    ],
    keys: &[],
    props: &[
        &["tone", "Tone", "Neutral (default), Success, Warning, Danger, Info or Accent"],
        &["dot", "bool", "Show a dot before the text. Default false"],
        &["children", "View", "The label"],
    ],
};
