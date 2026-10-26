use dioxus::prelude::*;
use oxide_dioxus::shortcut_recorder::ShortcutRecorder;

#[component]
pub fn ShortcutRecorderDemo() -> Element {
    // Stored as "Mod+Shift+K": Mod is ⌘ on a Mac and Ctrl elsewhere.
    let mut palette = use_signal(|| Some(String::from("Mod+K")));

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 420px;",
            div { style: "display: grid; gap: var(--space-2);",
                ShortcutRecorder {
                    label: "Open command palette",
                    value: Some(palette().unwrap_or_default()),
                    on_change: move |v: Option<String>| palette.set(v),
                }
                p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);",
                    "Stored as: {palette().unwrap_or_else(|| \"nothing\".into())}"
                }
            }
            ShortcutRecorder { label: "New chat", default_value: "Mod+Shift+O" }
            ShortcutRecorder { label: "Toggle sidebar" }
            ShortcutRecorder { label: "Locked", default_value: "Mod+Alt+L", disabled: true }
        }
    }
}
