use hydroxy_leptos::menu::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (last, set_last) = signal(String::from("none yet"));

    view! {
        <div style="display: flex; align-items: center; gap: var(--space-5);">
            // Uncontrolled. Opens below the trigger, flips above when there is no room.
            <Menu
                items=vec![
                    MenuEntry::new("rename"),
                    MenuEntry::new("duplicate"),
                    MenuEntry::new("archive").disabled(),
                    MenuEntry::new("delete"),
                ]
                on_select=move |v| set_last.set(v)
            >
                <MenuTrigger>"Project actions"</MenuTrigger>
                <MenuContent label="Q3 customer research">
                    <MenuItem value="rename" hint="R">"Rename project"</MenuItem>
                    <MenuItem value="duplicate" hint="D">"Duplicate project"</MenuItem>
                    <MenuItem value="archive">"Archive project"</MenuItem>
                    <MenuItem value="delete">"Delete project"</MenuItem>
                </MenuContent>
            </Menu>
            <span>"Last action: " {last}</span>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "menu",
    name: "Menu",
    group: "Overlays",
    summary: "A list of actions that opens from a button. Arrow keys move through enabled items; Esc or a click outside closes it.",
    demo,
    leptos: include_str!("menu.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/menu.rs"),
    parts: &[
        &["Menu", "(none)", "Provides the machine. Renders no element"],
        &[
            "MenuTrigger",
            "trigger",
            "button: aria-haspopup=menu, aria-expanded, aria-controls, popovertarget, CSS anchor",
        ],
        &["MenuContent", "content, label", "role=menu, popover=auto, anchored with flip, data-side, optional label"],
        &["MenuItem", "item, item-label, item-hint", "button role=menuitem, tabindex=-1, data-highlighted, disabled"],
    ],
    keys: &[
        &["Enter / Space / ArrowDown", "On the trigger: opens and focuses the first enabled item"],
        &["ArrowUp", "On the trigger: opens and focuses the last enabled item"],
        &["ArrowDown / ArrowUp", "Next / previous enabled item. Wraps at the ends"],
        &["Home / End", "First / last enabled item"],
        &["Enter / Space", "Runs the focused item, closes, returns focus to the trigger"],
        &["Escape", "Closes and returns focus to the trigger"],
        &["Tab", "Closes and moves focus on"],
    ],
    props: &[
        &["items", "Vec<MenuEntry>", "Required. Order and disabled state"],
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled. Default false"],
        &["placement", "Placement", "Side and alignment. Default below, start-aligned"],
        &["on_open_change", "Callback<bool>", "Called when the menu opens or closes"],
        &["on_select", "Callback<String>", "Called with the value of the item that ran"],
        &["MenuContent label", "String", "Optional small heading above the items"],
        &["MenuItem hint", "String", "Optional right-aligned hint, such as a shortcut"],
        &["variant", "Option<ButtonVariant>", "On MenuTrigger: the button look. Default secondary"],
    ],
};
