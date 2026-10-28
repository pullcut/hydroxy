use dioxus::prelude::*;
use hydroxy_dioxus::context_menu::{ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger, MenuEntry};

#[component]
pub fn ContextMenuDemo() -> Element {
    let mut last = use_signal(|| String::from("none yet"));

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 420px;",
            ContextMenu {
                items: vec![
                    MenuEntry::new("open"),
                    MenuEntry::new("rename"),
                    MenuEntry::new("share").disabled(),
                    MenuEntry::new("delete"),
                ],
                on_select: move |v| last.set(v),
                // Right click, or focus it and press Shift+F10.
                ContextMenuTrigger {
                    div { style: "display: grid; place-items: center; height: 140px; border: 1px dashed var(--border-2); border-radius: var(--radius-md); color: var(--text-2);",
                        "Right-click the interview notes"
                    }
                }
                ContextMenuContent {
                    ContextMenuItem { value: "open", hint: "↵", "Open notes" }
                    ContextMenuItem { value: "rename", hint: "R", "Rename" }
                    ContextMenuItem { value: "share", "Share" }
                    ContextMenuItem { value: "delete", "Delete" }
                }
            }
            span { "Last action: {last}" }
        }
    }
}
