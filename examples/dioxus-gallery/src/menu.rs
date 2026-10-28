use dioxus::prelude::*;
use hydroxy_dioxus::menu::{Menu, MenuContent, MenuEntry, MenuItem, MenuTrigger};

#[component]
pub fn MenuDemo() -> Element {
    let mut last = use_signal(|| String::from("none yet"));

    rsx! {
        div { style: "display: flex; align-items: center; gap: var(--space-5);",
            // Uncontrolled. Opens below the trigger, flips above when there is no room.
            Menu {
                items: vec![
                    MenuEntry::new("rename"),
                    MenuEntry::new("duplicate"),
                    MenuEntry::new("archive").disabled(),
                    MenuEntry::new("delete"),
                ],
                on_select: move |v| last.set(v),
                MenuTrigger { "Project actions" }
                MenuContent { label: "Q3 customer research",
                    MenuItem { value: "rename", hint: "R", "Rename project" }
                    MenuItem { value: "duplicate", hint: "D", "Duplicate project" }
                    MenuItem { value: "archive", "Archive project" }
                    MenuItem { value: "delete", "Delete project" }
                }
            }
            span { "Last action: {last}" }
        }
    }
}
