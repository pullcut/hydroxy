use dioxus::prelude::*;
use hydroxy_dioxus::markdown_editor::MarkdownEditor;

#[component]
pub fn MarkdownEditorDemo() -> Element {
    let mut notes = use_signal(|| {
        String::from(
            "## Standup\n\nShipped the **export** fix. Next: _pricing_ page.\n\n- [x] Review PR\n- [ ] Write release notes",
        )
    });

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 560px;",
            MarkdownEditor {
                label: "Notes",
                value: notes(),
                on_input: move |v| notes.set(v),
                placeholder: "Write in markdown",
            }
            p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);",
                "{notes().chars().count()} characters"
            }
        }
    }
}
