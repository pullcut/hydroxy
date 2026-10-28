use hydroxy_leptos::value_diff::ValueDiff;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    view! {
        <ul style="list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);">
            <li style="display: flex; align-items: center; gap: 10px; font: var(--text-sm);">
                "Jo Marsh moved the status" <ValueDiff from="In review" to="Ready" />
            </li>
            <li style="display: flex; align-items: center; gap: 10px; font: var(--text-sm);">
                "Amara Kaur changed the due date" <ValueDiff from="24 Oct" to="31 Oct" />
            </li>
        </ul>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "value-diff",
    name: "Value diff",
    group: "Display",
    summary: "Before → after for activity feeds. The old value is struck through; screen readers hear \"changed from … to …\".",
    demo,
    leptos: include_str!("value_diff.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/value_diff.rs"),
    parts: &[
        &["ValueDiff", "—", "span[data-scope=value-diff]"],
        &["", "from", "del: the old value, struck through"],
        &["", "to", "ins: the new value"],
    ],
    keys: &[],
    props: &[&["from", "String", "The old value"], &["to", "String", "The new value"]],
};
