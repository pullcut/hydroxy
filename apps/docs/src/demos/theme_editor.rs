use leptos::prelude::*;
use oxide_leptos::theme_editor::ThemeEditor;

pub fn demo() -> AnyView {
    let (edited, set_edited) = signal(0usize);

    view! {
        <div style="display: grid; gap: var(--space-5);">
            // Start from a preset, edit tokens, watch the preview and contrast checks, then export.
            <ThemeEditor default_preset="light" on_change=move |_tokens| set_edited.update(|n| *n += 1) />
            <span style="font: var(--text-sm); color: var(--text-2);">{move || format!("{} edits", edited.get())}</span>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "theme-editor",
    name: "Theme editor",
    group: "Forms",
    summary: "Edit oxide's design tokens live on top of a preset: colours, corner radii, spacing scale and fonts, with a scoped preview, WCAG contrast warnings and CSS or JSON export.",
    demo,
    leptos: include_str!("theme_editor.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/theme_editor.rs"),
    parts: &[
        &["ThemeEditor", "root", "role=group, the controls beside the output"],
        &["ThemeEditor", "controls / toolbar / section", "Preset select, Reset all, then Colours, Corners, Spacing and Type"],
        &["ThemeEditor", "row", "One token. data-modified when it differs from the preset"],
        &["ThemeEditor", "swatch / name / value", "Inside the colour row's popover trigger; the popover holds a ColorPicker"],
        &["ThemeEditor", "warning", "Contrast ratio when the token fails a check it is the text of"],
        &["ThemeEditor", "reset", "Puts one token back to the preset. Disabled when unchanged"],
        &["ThemeEditor", "preview", "role=region with the preset's data-theme and the edited tokens inline, so edits stay inside it"],
        &["ThemeEditor", "contrast / live / issues", "role=status summary of WCAG AA checks, then the failing pairs"],
        &["ThemeEditor", "export", "CSS or JSON choice and a code block with a copy button"],
    ],
    keys: &[
        &["Enter / Space (colour row)", "Open the colour picker for that token"],
        &["Escape (picker)", "Close it and return to the row"],
        &["Arrows (sliders)", "Change a radius or the spacing scale"],
        &["Tab", "Move between tokens and their reset buttons"],
    ],
    props: &[
        &["default_preset", "String", "Preset to start from, as a data-theme value. Default light"],
        &["on_change", "Callback<Vec<(String, String)>>", "Every token (without --) and its value after each change"],
    ],
};
