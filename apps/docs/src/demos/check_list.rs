use hydroxy_leptos::check_list::{Check, CheckList, CheckState};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let finished = vec![
        Check::new("Lint", CheckState::Pass),
        Check::new("Unit tests", CheckState::Pass),
        Check::new("Accessibility", CheckState::Fail),
        Check::new("Visual diff", CheckState::Skip),
    ];
    let running = vec![
        Check::new("Lint", CheckState::Pass),
        Check::new("Unit tests", CheckState::Running),
        Check::new("Accessibility", CheckState::Running),
    ];
    view! {
        <div style="display: grid; gap: var(--space-5); max-width: 420px;">
            <CheckList checks=finished />
            <CheckList checks=running />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "check-list",
    name: "Check list",
    group: "Feedback",
    summary: "Automated check results with a summary line. Failed and skipped rows say so in words, and running checks breathe.",
    demo,
    leptos: include_str!("check_list.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/check_list.rs"),
    parts: &[
        &["CheckList", "summary", "Overall state and \"4 of 6 checks passed\", announced politely"],
        &["", "row", "li with data-state=pass|fail|skip|running"],
        &["", "mark", "Tick, cross, empty ring, or a breathing ring when running"],
    ],
    keys: &[],
    props: &[
        &["checks", "Vec<Check>", "Required. Check::new(name, CheckState)"],
        &["summary_text", "String", "Replaces the generated summary line"],
    ],
};
