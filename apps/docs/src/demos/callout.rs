use hydroxy_leptos::callout::{Callout, CalloutKind};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="display: grid; gap: var(--space-4);">
            <Callout title="Sources are attached">
                "Each answer lists the files it read, with the page for each figure."
            </Callout>
            <Callout kind=CalloutKind::Tip title="Compare two exports">
                "Drop both files in and ask what changed between them."
            </Callout>
            <Callout kind=CalloutKind::Warning title="Two items need your judgement">
                "The September export has 410 fewer tickets than the August summary."
            </Callout>
            <Callout kind=CalloutKind::Danger title="Deleting removes 12 files">
                "Summaries that cite them lose their sources. This can't be undone."
            </Callout>
            <Callout>"A callout can be a single line with no title."</Callout>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "callout",
    name: "Callout",
    group: "Feedback",
    summary: "A tinted box for notes, tips, warnings and dangers. Each kind has a fixed colour and icon.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("callout.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/callout.rs"),
    parts: &[
        &["Callout", "root", "<div role=note>, or role=alert for a live danger callout. data-kind"],
        &["(inside)", "icon", "The kind's icon, 18px"],
        &["(inside)", "title", "Optional first line"],
        &["(inside)", "body", "The children"],
    ],
    keys: &[],
    props: &[
        &["kind", "CalloutKind", "Note (sapphire), Tip (emerald), Warning (citrine) or Danger (garnet). Default Note"],
        &["title", "Option<String>", "Optional first line"],
        &["live", "bool", "Announce a danger callout when it appears (role=alert)"],
        &["children", "Children", "The body"],
    ],
    events: &[],
};
