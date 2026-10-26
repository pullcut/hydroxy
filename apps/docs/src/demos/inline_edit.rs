use leptos::prelude::*;
use oxide_leptos::inline_edit::InlineEdit;

pub fn demo() -> AnyView {
    let (name, set_name) = signal(String::from("Q3 customer interviews"));

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 420px;">
            // Controlled: the owner keeps the committed value.
            <div style="display: grid; gap: var(--space-2);">
                <InlineEdit label="Project name" value=name on_change=move |v| set_name.set(v) />
                <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">"Saved as: " {name}</p>
            </div>

            // Uncontrolled, with a placeholder for the empty state.
            <InlineEdit label="Description" placeholder="Add a description" />

            // A textarea. Enter adds a line; Ctrl or ⌘ + Enter commits.
            <InlineEdit
                label="Notes"
                multiline=true
                default_value="Interviews run Monday to Thursday.\nSynthesis on Friday."
            />

            <InlineEdit label="Locked title" default_value="Archived project" disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "inline-edit",
    name: "Inline edit",
    group: "Forms",
    summary: "Text that becomes a field when you click it. Enter or leaving the field saves, Esc puts the old text back.",
    demo,
    leptos: include_str!("inline_edit.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/inline_edit.rs"),
    parts: &[
        &["InlineEdit", "root", "Wrapper. data-editing while the field shows, data-disabled, data-multiline"],
        &["InlineEdit", "preview", "<button> showing the text, or the placeholder (data-empty). Named \"Edit <label>\""],
        &["InlineEdit", "field", "Native <input> or <textarea>, named by the label. Hidden until editing"],
    ],
    keys: &[
        &["Enter / F2", "On the text: start editing"],
        &["Enter", "In the field: save and return focus to the text"],
        &["Ctrl or ⌘ + Enter", "In a textarea: save"],
        &["Escape", "Discard the draft and return focus to the text"],
        &["Tab / click away", "Save, and keep going where you went"],
    ],
    props: &[
        &["label", "String", "Required. Names the button and the field"],
        &["value", "Option<String>", "Controlled value. Pair with on_change"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["on_change", "Callback<String>", "Called on save, only if the text changed"],
        &["placeholder", "String", "Shown dimmed while the value is empty"],
        &["multiline", "bool", "Use a textarea. Enter adds a line"],
        &["disabled", "bool", "Block editing"],
    ],
};
