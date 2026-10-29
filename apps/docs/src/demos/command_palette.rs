use hydroxy_leptos::button::{Button, ButtonVariant};
use hydroxy_leptos::command_palette::*;
use hydroxy_leptos::icon::IconName;
use hydroxy_leptos::kbd::Kbd;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (open, set_open) = signal(false);
    let (last, set_last) = signal(String::from("none yet"));

    // ⌘K or Ctrl+K toggles the palette anywhere on the page.
    let shortcut = window_event_listener(leptos::ev::keydown, move |ev| {
        if (ev.meta_key() || ev.ctrl_key()) && ev.key().eq_ignore_ascii_case("k") {
            ev.prevent_default();
            set_open.update(|o| *o = !*o);
        }
    });
    on_cleanup(move || shortcut.remove());

    let commands = vec![
        CommandItem::new("new", "New project", "Projects").icon(IconName::Plus).hint("N"),
        CommandItem::new("open", "Open Q3 customer research", "Projects").icon(IconName::FileText),
        CommandItem::new("summary", "Draft a summary", "Actions").icon(IconName::Lightbulb).keywords("write report"),
        CommandItem::new("copy", "Copy share link", "Actions").icon(IconName::Copy).hint("C"),
        CommandItem::new("archive", "Archive project", "Actions").icon(IconName::Inbox),
        CommandItem::new("search", "Search sources", "Navigate").icon(IconName::Search).hint("/"),
    ];

    view! {
        <div style="display: flex; align-items: center; gap: var(--space-5); flex-wrap: wrap;">
            <Button variant=ButtonVariant::Secondary icon_left=IconName::Search on_click=move |_| set_open.set(true)>
                "Search or run a command"
            </Button>
            <span><Kbd>"⌘"</Kbd> " " <Kbd>"K"</Kbd></span>
            <span>"Last command: " {last}</span>
        </div>
        // Controlled: the page owns `open`, so the button and the shortcut can both drive it.
        <CommandPalette
            items=commands
            open=open
            on_open_change=move |o| set_open.set(o)
            on_select=move |id| set_last.set(id)
        />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "command-palette",
    name: "Command palette",
    group: "Overlays",
    summary: "A modal search over every command, grouped. Type to filter, Enter to run. Open it with a button or ⌘K.",
    demo,
    leptos: include_str!("command_palette.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/command_palette.rs"),
    parts: &[
        &["CommandPalette", "content, panel", "Native <dialog> opened with showModal(); the panel sits near the top"],
        &["(search)", "search, input", "Search icon and input role=combobox with aria-activedescendant"],
        &[
            "(results)",
            "list, group, group-label, item, item-icon, item-label, item-hint, empty",
            "role=listbox, role=group per heading, role=option per command, data-highlighted",
        ],
    ],
    keys: &[
        &["⌘K / Ctrl+K", "Opens or closes it (wired by the page, as in this demo)"],
        &["Typing", "Filters by label, group and keywords. Highlights the first result"],
        &["ArrowDown / ArrowUp", "Next / previous command. Wraps at the ends"],
        &["Enter", "Runs the highlighted command and closes"],
        &["Escape", "Closes"],
    ],
    props: &[
        &["items", "Vec<CommandItem>", "Required. id, label, group, plus optional keywords, hint and icon"],
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled. Default false"],
        &["placeholder", "String", "Default \"Search or run a command…\""],
        &["label", "String", "Accessible name. Default \"Command palette\""],
        &["on_open_change", "Callback<bool>", "Called when it opens or closes, or a change is requested"],
        &["on_select", "Callback<String>", "Called with the id of the command that ran"],
    ],
    events: &[],
};
