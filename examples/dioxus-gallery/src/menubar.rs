use dioxus::prelude::*;
use oxide_dioxus::menubar::{MenuEntry, Menubar, MenubarEntry, MenubarItem, MenubarMenu};

#[component]
pub fn MenubarDemo() -> Element {
    let mut last = use_signal(|| String::from("none yet"));

    rsx! {
        div { style: "display: grid; gap: var(--space-4); justify-items: start;",
            // Uncontrolled. Once one menu is open, hovering or arrowing to another switches to it.
            Menubar {
                label: "Document",
                menus: vec![MenubarEntry::new("file"), MenubarEntry::new("edit"), MenubarEntry::new("view")],
                MenubarMenu {
                    value: "file",
                    label: "File",
                    items: vec![
                        MenuEntry::new("new"),
                        MenuEntry::new("open"),
                        MenuEntry::new("export").disabled(),
                        MenuEntry::new("print"),
                    ],
                    on_select: move |v| last.set(v),
                    MenubarItem { value: "new", hint: "N", "New document" }
                    MenubarItem { value: "open", hint: "O", "Open" }
                    MenubarItem { value: "export", "Export" }
                    MenubarItem { value: "print", hint: "P", "Print" }
                }
                MenubarMenu {
                    value: "edit",
                    label: "Edit",
                    items: vec![MenuEntry::new("undo"), MenuEntry::new("redo"), MenuEntry::new("find")],
                    on_select: move |v| last.set(v),
                    MenubarItem { value: "undo", hint: "Z", "Undo" }
                    MenubarItem { value: "redo", hint: "Y", "Redo" }
                    MenubarItem { value: "find", hint: "F", "Find" }
                }
                MenubarMenu {
                    value: "view",
                    label: "View",
                    items: vec![MenuEntry::new("zoom-in"), MenuEntry::new("zoom-out"), MenuEntry::new("fullscreen")],
                    on_select: move |v| last.set(v),
                    MenubarItem { value: "zoom-in", "Zoom in" }
                    MenubarItem { value: "zoom-out", "Zoom out" }
                    MenubarItem { value: "fullscreen", "Full screen" }
                }
            }
            span { "Last action: {last}" }
        }
    }
}
