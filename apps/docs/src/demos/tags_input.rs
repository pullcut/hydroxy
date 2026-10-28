use hydroxy_leptos::tags_input::TagsInput;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (topics, set_topics) = signal(vec!["onboarding".to_string(), "billing".to_string()]);

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 480px;">
            // Controlled, with a limit and a form name.
            <TagsInput
                label="Topics"
                value=topics
                on_value_change=move |tags| set_topics.set(tags)
                max=5
                name="topics"
                placeholder="Add a topic"
            />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || format!("Tagged: {}", topics.get().join(", "))}
            </p>

            // Uncontrolled.
            <TagsInput label="Notify these people" placeholder="Type a name, then press Enter" />

            // Disabled.
            <TagsInput label="Shared with" default_value=vec!["Jo Marsh".to_string(), "Ana Kowal".to_string()] disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tags-input",
    name: "Tags input",
    group: "Forms",
    summary: "A list of short values typed into one field. Enter or a comma adds a tag; pasted lists are split on commas and new lines.",
    demo,
    leptos: include_str!("tags_input.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tags_input.rs"),
    parts: &[
        &["TagsInput", "root", "Wraps everything. data-disabled"],
        &["(inside)", "label", "<label for> the draft input"],
        &["(inside)", "field", "The bordered box. A click anywhere focuses the draft"],
        &["(inside)", "tag", "One per value. data-highlighted while Backspace or the arrows pick it"],
        &["(inside)", "remove", "<button aria-label=\"Remove {tag}\">, out of the tab order"],
        &["(inside)", "control", "The draft <input>, aria-describedby the counter"],
        &["(inside)", "counter", "\"3 of 5\" when max is set"],
        &["(inside)", "live", "Visually hidden aria-live region that announces adds and removals"],
    ],
    keys: &[
        &["Enter / ,", "Add the draft as a tag"],
        &["Backspace", "With an empty draft: highlight the last tag, then remove it"],
        &["ArrowLeft / ArrowRight", "With an empty draft: move the highlight; past the last tag returns to typing"],
        &["Delete", "Remove the highlighted tag"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<Vec<String>>", "Controlled tags. Pair with on_value_change"],
        &["default_value", "Vec<String>", "Initial tags when uncontrolled"],
        &["max", "Option<usize>", "Most tags allowed. Shows a counter"],
        &["disabled", "bool", "Block edits"],
        &["name", "String", "Form field name. Each tag submits as its own value"],
        &["placeholder", "String", "Placeholder for the draft input"],
        &["on_value_change", "Callback<Vec<String>>", "Called with the new tags"],
    ],
};
