use dioxus::prelude::*;
use oxide_dioxus::theme_editor::ThemeEditor;

#[component]
pub fn ThemeEditorDemo() -> Element {
    let mut edited = use_signal(|| 0usize);

    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            // Start from a preset, edit tokens, watch the preview and contrast checks, then export.
            ThemeEditor { default_preset: "light", on_change: move |_tokens| edited += 1 }
            span { style: "font: var(--text-sm); color: var(--text-2);", "{edited} edits" }
        }
    }
}
