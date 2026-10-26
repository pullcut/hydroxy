use leptos::prelude::*;
use oxide_leptos::context_menu::*;

pub fn demo() -> AnyView {
    let (last, set_last) = signal(String::from("none yet"));

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 420px;">
            <ContextMenu
                items=vec![
                    MenuEntry::new("open"),
                    MenuEntry::new("rename"),
                    MenuEntry::new("share").disabled(),
                    MenuEntry::new("delete"),
                ]
                on_select=move |v| set_last.set(v)
            >
                // Right click, or focus it and press Shift+F10.
                <ContextMenuTrigger>
                    <div style="display: grid; place-items: center; height: 140px; border: 1px dashed var(--border-2); border-radius: var(--radius-md); color: var(--text-2);">
                        "Right-click the interview notes"
                    </div>
                </ContextMenuTrigger>
                <ContextMenuContent>
                    <ContextMenuItem value="open" hint="↵">"Open notes"</ContextMenuItem>
                    <ContextMenuItem value="rename" hint="R">"Rename"</ContextMenuItem>
                    <ContextMenuItem value="share">"Share"</ContextMenuItem>
                    <ContextMenuItem value="delete">"Delete"</ContextMenuItem>
                </ContextMenuContent>
            </ContextMenu>
            <span>"Last action: " {last}</span>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "context-menu",
    name: "Context menu",
    group: "Overlays",
    summary: "A menu of actions that opens where you right-click. The same keyboard model as Menu, opened from the keyboard with Shift+F10 or the menu key.",
    demo,
    leptos: include_str!("context_menu.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/context_menu.rs"),
    parts: &[
        &["ContextMenu", "(none)", "Provides the machine. Renders no element"],
        &["ContextMenuTrigger", "trigger", "Focusable area that takes the right click. data-state"],
        &["ContextMenuContent", "anchor", "Zero-size fixed point at the pointer, the CSS anchor"],
        &["", "content", "role=menu, popover=auto, anchored to the point, flips at the viewport edges"],
        &["ContextMenuItem", "item, item-label, item-hint", "button role=menuitem, tabindex=-1, data-highlighted, disabled"],
    ],
    keys: &[
        &["Shift+F10 / ContextMenu", "On the trigger: opens at the focused element"],
        &["ArrowDown / ArrowUp", "Next / previous enabled item. Wraps at the ends"],
        &["Home / End", "First / last enabled item"],
        &["Enter / Space", "Runs the focused item and closes"],
        &["Escape", "Closes and returns focus"],
        &["Tab", "Closes and moves focus on"],
    ],
    props: &[
        &["items", "Vec<MenuEntry>", "Required. Order and disabled state"],
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["on_open_change", "Callback<bool>", "Called when the menu opens or closes"],
        &["on_select", "Callback<String>", "Called with the value of the item that ran"],
        &["ContextMenuItem hint", "String", "Optional right-aligned hint, such as a shortcut"],
    ],
};
