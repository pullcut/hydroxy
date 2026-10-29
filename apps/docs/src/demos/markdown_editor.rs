use hydroxy_leptos::markdown_editor::MarkdownEditor;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (notes, set_notes) = signal(String::from(
        "## Standup\n\nShipped the **export** fix. Next: _pricing_ page.\n\n- [x] Review PR\n- [ ] Write release notes",
    ));

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 560px;">
            <MarkdownEditor
                label="Notes"
                value=notes
                on_input=move |v| set_notes.set(v)
                placeholder="Write in markdown"
            />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                {move || format!("{} characters", notes.get().chars().count())}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "markdown-editor",
    name: "Markdown editor",
    group: "Forms",
    summary: "A textarea with a formatting toolbar and Write / Preview tabs. Toolbar buttons and shortcuts wrap or prefix the selection, and toggle off when it is already formatted.",
    demo,
    leptos: include_str!("markdown_editor.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/markdown_editor.rs"),
    parts: &[
        &["MarkdownEditor", "root", "Wrapper"],
        &["", "label", "<label for> the textarea"],
        &["", "bar", "Tabs list and toolbar row"],
        &["", "toolbar", "role=toolbar of action buttons. Hidden in Preview"],
        &["", "action", "One formatting button. data-format names it"],
        &["", "control", "The native <textarea>"],
        &["", "preview", "Rendered Markdown of the current text"],
    ],
    keys: &[&["Cmd/Ctrl + B", "Bold"], &["Cmd/Ctrl + I", "Italic"], &["Cmd/Ctrl + K", "Link, with the url selected"]],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<String>", "Controlled value. Pair with on_input"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["on_input", "Callback<String>", "Called on every change, typed or from the toolbar"],
        &["placeholder", "String", "Placeholder text"],
        &["name", "String", "Form field name"],
        &["rows", "u32", "Visible text rows. Default 8"],
    ],
    events: &[],
};
