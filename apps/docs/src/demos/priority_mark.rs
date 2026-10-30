use hydroxy_leptos::priority_mark::PriorityMark;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let tasks = [
        (0, "Fix the September export count"),
        (1, "Review two flagged summaries"),
        (2, "Tag the Q3 interviews"),
        (3, "Tidy the archive"),
    ];
    view! {
        <ul style="list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);">
            {tasks
                .into_iter()
                .map(|(level, task)| {
                    view! {
                        <li style="display: flex; align-items: center; gap: 10px; font: var(--text-sm);">
                            <PriorityMark level=level />
                            {task}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "priority-mark",
    name: "Priority mark",
    group: "Display",
    summary: "P0 to P3 in mono. P0 is solid garnet so urgent work stands out in a dense list.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("priority_mark.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/priority_mark.rs"),
    parts: &[&["PriorityMark", "—", "span[data-scope=priority-mark][data-level]"]],
    keys: &[],
    props: &[
        &["level", "u8", "0 (urgent) to 3 (low). Default 2"],
        &["label", "String", "Overrides the name: Urgent, High, Medium, Low"],
    ],
    events: &[],
};
