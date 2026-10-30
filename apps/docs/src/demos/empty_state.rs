use hydroxy_leptos::empty_state::EmptyState;
use hydroxy_leptos::icon::IconName;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="display: grid; gap: var(--space-5);">
            <EmptyState
                title="Nothing archived yet"
                description="Archived projects show up here. Archiving keeps their files and summaries."
            >
                <a href="#/button">"Open the project list"</a>
            </EmptyState>

            <div style="border: 1px dashed var(--border-2); border-radius: var(--radius-lg);">
                <EmptyState
                    icon=IconName::Search
                    title="No matches"
                    description="Nothing in this workspace mentions \"refund policy\"."
                />
            </div>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "empty-state",
    name: "Empty state",
    group: "Feedback",
    summary: "Fills an empty list or pane: an icon, what is empty, one line on why, and an optional action.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("empty_state.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/empty_state.rs"),
    parts: &[
        &["EmptyState", "root", "<div>, centred column"],
        &["(inside)", "icon", "40px icon in a 64px sunken tile"],
        &["(inside)", "title", "What is empty"],
        &["(inside)", "description", "Optional line, max 36 characters wide"],
        &["(inside)", "action", "The children, when given"],
    ],
    keys: &[],
    props: &[
        &["icon", "IconName", "Default Inbox"],
        &["title", "String", "Required. Such as \"Nothing archived yet\""],
        &["description", "Option<String>", "One line on why, or what fills it"],
        &["children", "Children", "Optional action, such as a button"],
    ],
    events: &[],
};
