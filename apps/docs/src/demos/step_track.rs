use leptos::prelude::*;
use hydroxy_leptos::step_track::{Jewel, StepTrack};

pub fn demo() -> AnyView {
    let steps = || vec!["Draft".to_string(), "In review".into(), "Approved".into(), "Shared".into()];
    view! {
        <div style="display: grid; gap: var(--space-8);">
            <StepTrack steps=steps() current=1 label="Summary progress" />
            <StepTrack steps=steps() current=3 jewel=Jewel::Emerald label="Release progress" />
            <StepTrack steps=steps() label="Not started" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "step-track",
    name: "Step track",
    group: "Data",
    summary: "A linear stage tracker with diamond nodes, for approvals, deploys or onboarding. Filled up to the current step.",
    demo,
    leptos: include_str!("step_track.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/step_track.rs"),
    parts: &[
        &["StepTrack", "—", "ol[data-scope=step-track]"],
        &["", "step", "li; aria-current=step on the current one, \"done\" for screen readers"],
        &["", "dot", "Diamond node, filled once reached"],
        &["", "line", "Connector, filled once reached"],
    ],
    keys: &[],
    props: &[
        &["steps", "Vec<String>", "Required. Step names in order"],
        &["current", "usize", "Index of the current step. None: nothing reached"],
        &["jewel", "Jewel", "Fill colour. Default Sapphire"],
        &["label", "String", "Accessible name. Default Progress"],
    ],
};
