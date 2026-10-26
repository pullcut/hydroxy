use leptos::prelude::*;
use oxide_leptos::insight_note::InsightNote;

pub fn demo() -> AnyView {
    view! {
        <div style="display: grid; gap: var(--space-5); max-width: 520px;">
            <InsightNote sources=vec!["interviews-q3.pdf".to_string(), "tickets-sept.csv".to_string()]>
                <p>
                    "Nine of twelve interviewees raised onboarding. The September export has 410 fewer tickets than the August summary, so two figures need checking."
                </p>
            </InsightNote>
            <InsightNote label="Weekly digest">
                <p>"Three projects moved to ready. One is waiting on a review from Amara Kaur."</p>
            </InsightNote>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "insight-note",
    name: "Insight note",
    group: "Chat",
    summary: "An amethyst-tinted summary with the mark, a label and the sources it came from. Say where each figure comes from.",
    demo,
    leptos: include_str!("insight_note.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/insight_note.rs"),
    parts: &[
        &["InsightNote", "—", "section labelled by `label`"],
        &["", "head", "The mark, the label and an optional action"],
        &["", "body", "Your content"],
        &["", "sources", "SourceRow of SourceChips"],
    ],
    keys: &[],
    props: &[
        &["label", "String", "What the note is. Default Summary"],
        &["sources", "Vec<String>", "File names shown as source chips"],
        &["action", "View", "Control at the end of the header"],
        &["children", "View", "The note"],
    ],
};
