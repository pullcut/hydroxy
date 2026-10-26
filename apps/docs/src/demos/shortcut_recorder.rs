use leptos::prelude::*;
use oxide_leptos::shortcut_recorder::ShortcutRecorder;

pub fn demo() -> AnyView {
    // Stored as "Mod+Shift+K": Mod is ⌘ on a Mac and Ctrl elsewhere.
    let (palette, set_palette) = signal(Some(String::from("Mod+K")));

    let save = move |v: Option<String>| set_palette.set(v);

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 420px;">
            // Controlled.
            <div style="display: grid; gap: var(--space-2);">
                <ShortcutRecorder
                    label="Open command palette"
                    value=Signal::derive(move || Some(palette.get().unwrap_or_default()))
                    on_change=save
                />
                <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                    "Stored as: " {move || palette.get().unwrap_or_else(|| "nothing".into())}
                </p>
            </div>

            // Uncontrolled.
            <ShortcutRecorder label="New chat" default_value="Mod+Shift+O" />
            <ShortcutRecorder label="Toggle sidebar" />
            <ShortcutRecorder label="Locked" default_value="Mod+Alt+L" disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "shortcut-recorder",
    name: "Shortcut recorder",
    group: "Forms",
    summary: "Click it and press a key combination to record a shortcut. It is stored in a portable form and shown with the right modifier symbols for the platform.",
    demo,
    leptos: include_str!("shortcut_recorder.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/shortcut_recorder.rs"),
    parts: &[
        &["ShortcutRecorder", "root", "Wrapper. data-recording while listening, data-disabled"],
        &["ShortcutRecorder", "label", "Names the control"],
        &["ShortcutRecorder", "control", "<button> that listens. data-recording while it does"],
        &["ShortcutRecorder", "keys, placeholder", "The recorded combo as Kbd keycaps, or the empty text"],
        &["IconButton", "clear", "Removes the shortcut. Shown when one is set"],
        &["ShortcutRecorder", "status", "role=status text. Visible while recording, so it can explain a refused key"],
    ],
    keys: &[
        &["Enter / Space", "On the control: start listening"],
        &["Any combination", "While listening: record it. Needs Ctrl, Alt, ⌘ or a function key"],
        &["Escape", "While listening: cancel and keep the old shortcut"],
        &["Backspace / Delete", "While listening: remove the shortcut"],
        &["Tab", "Leave the control and stop listening"],
    ],
    props: &[
        &["label", "String", "Required. What the shortcut is for"],
        &["value", "Option<String>", "Controlled shortcut such as Mod+K. Empty string means none. Pair with on_change"],
        &["default_value", "String", "Initial shortcut when uncontrolled"],
        &["on_change", "Callback<Option<String>>", "The new shortcut, or None when cleared"],
        &["platform", "Platform", "Mac or Other. Default: detected after mount"],
        &["placeholder", "String", "Text when empty. Default \"Not set\""],
        &["disabled", "bool", "Block recording and clearing"],
    ],
};
