use hydroxy_leptos::menubar::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (last, set_last) = signal(String::from("none yet"));

    view! {
        <div style="display: grid; gap: var(--space-4); justify-items: start;">
            // Uncontrolled. Once one menu is open, hovering or arrowing to another switches to it.
            <Menubar
                label="Document"
                menus=vec![MenubarEntry::new("file"), MenubarEntry::new("edit"), MenubarEntry::new("view")]
            >
                <MenubarMenu
                    value="file"
                    label="File"
                    items=vec![
                        MenuEntry::new("new"),
                        MenuEntry::new("open"),
                        MenuEntry::new("export").disabled(),
                        MenuEntry::new("print"),
                    ]
                    on_select=move |v| set_last.set(v)
                >
                    <MenubarItem value="new" hint="N">"New document"</MenubarItem>
                    <MenubarItem value="open" hint="O">"Open"</MenubarItem>
                    <MenubarItem value="export">"Export"</MenubarItem>
                    <MenubarItem value="print" hint="P">"Print"</MenubarItem>
                </MenubarMenu>
                <MenubarMenu
                    value="edit"
                    label="Edit"
                    items=vec![MenuEntry::new("undo"), MenuEntry::new("redo"), MenuEntry::new("find")]
                    on_select=move |v| set_last.set(v)
                >
                    <MenubarItem value="undo" hint="Z">"Undo"</MenubarItem>
                    <MenubarItem value="redo" hint="Y">"Redo"</MenubarItem>
                    <MenubarItem value="find" hint="F">"Find"</MenubarItem>
                </MenubarMenu>
                <MenubarMenu
                    value="view"
                    label="View"
                    items=vec![MenuEntry::new("zoom-in"), MenuEntry::new("zoom-out"), MenuEntry::new("fullscreen")]
                    on_select=move |v| set_last.set(v)
                >
                    <MenubarItem value="zoom-in">"Zoom in"</MenubarItem>
                    <MenubarItem value="zoom-out">"Zoom out"</MenubarItem>
                    <MenubarItem value="fullscreen">"Full screen"</MenubarItem>
                </MenubarMenu>
            </Menubar>
            <span>"Last action: " {last}</span>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "menubar",
    name: "Menubar",
    group: "Navigation",
    summary: "A horizontal bar of menus, like File and Edit in a desktop app. One menu is open at a time; Left and Right move between them, and hovering switches while one is open.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("menubar.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/menubar.rs"),
    parts: &[
        &["Menubar", "root", "div, role=menubar. Owns which menu is open"],
        &[
            "MenubarMenu",
            "trigger",
            "button, role=menuitem, aria-haspopup=menu, aria-expanded. One tab stop for the whole bar",
        ],
        &["", "content", "popover=auto, role=menu. Styled like Menu (data-scope=menu)"],
        &["MenubarItem", "item", "button, role=menuitem. Highlighted item has data-highlighted"],
    ],
    keys: &[
        &[
            "Left / Right",
            "Moves to the previous or next menu, wrapping. While a menu is open, opens that menu instead",
        ],
        &["Down / Up", "Opens the menu on its first item"],
        &["Enter / Space", "Opens the menu, or activates the focused item"],
        &["Home / End", "Focuses the first or last menu in the bar"],
        &["Escape", "Closes the open menu and returns focus to its trigger"],
        &["Tab", "Closes the open menu and leaves the bar"],
    ],
    props: &[
        &["menus", "Vec<MenubarEntry>", "On Menubar: the menus in order. Disabled ones are skipped"],
        &["label", "String", "On Menubar: accessible name. Default Menu bar"],
        &["open", "Option<Option<String>>", "On Menubar: controlled open menu. Pair with on_open_change"],
        &["default_open", "Option<String>", "On Menubar: initially open menu when uncontrolled"],
        &["on_open_change", "Callback<Option<String>>", "On Menubar: called with the open menu, or None"],
        &["value", "String", "On MenubarMenu and MenubarItem: the key. Must match an entry"],
        &["label", "String", "On MenubarMenu: text of the trigger"],
        &["items", "Vec<MenuEntry>", "On MenubarMenu: the items in order. Disabled ones are skipped"],
        &["on_select", "Callback<String>", "On MenubarMenu: called with the activated item's value"],
        &["hint", "Option<String>", "On MenubarItem: right-aligned hint, such as a shortcut"],
    ],
    events: &[],
};
