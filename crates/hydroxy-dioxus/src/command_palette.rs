//! `CommandPalette`: a modal search over commands, on a native `<dialog>`
//! opened with `showModal()`. Focus stays in the search input; the highlight is
//! announced with `aria-activedescendant`.

use dioxus::prelude::*;
use hydroxy_headless::command_palette::{
    self as h, CommandPaletteCommand, CommandPaletteEvent, CommandPaletteInput, CommandPaletteMachine,
};
use hydroxy_statechart::Machine;

use crate::combobox::scroll_into_list;
use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::overlay::{close_dialog, show_modal};

pub use hydroxy_headless::command_palette::CommandItem;

/// A command palette. Open it from a button or a shortcut; Enter runs the
/// highlighted command and closes, Esc or a press outside closes.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn CommandPalette(
    /// The commands. Shown grouped, groups in first-appearance order.
    items: ReadSignal<Vec<CommandItem>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Search placeholder.
    #[props(default = "Search or run a command…".to_string())]
    placeholder: String,
    /// Accessible name for the dialog and list.
    #[props(default = "Command palette".to_string())]
    label: String,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    /// Called with the id of the command that ran.
    on_select: Option<EventHandler<String>>,
) -> Element {
    let input =
        use_memo(move || CommandPaletteInput { entries: h::entries(&items.read()), open: open(), default_open });
    let mut state = use_signal(|| CommandPaletteMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CommandPaletteMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    // The native dialog follows the state; keep the highlighted command in view.
    {
        let root = root.clone();
        use_effect(move || {
            let (open, highlighted) = {
                let s = state.read();
                (s.open.value, s.highlighted.clone())
            };
            if open {
                show_modal(&h::content_id(&root));
                if let Some(item) = highlighted {
                    scroll_into_list(&h::item_id(&root, &item));
                }
            } else {
                close_dialog(&h::content_id(&root));
            }
        });
    }

    let send = move |event: CommandPaletteEvent| {
        for command in drive::<CommandPaletteMachine>(state, input, event) {
            match command {
                CommandPaletteCommand::OpenChange(open) => {
                    if let Some(handler) = on_open_change {
                        handler.call(open);
                    }
                }
                CommandPaletteCommand::Select(item) => {
                    if let Some(handler) = on_select {
                        handler.call(item);
                    }
                }
            }
        }
    };

    let (query, highlighted, open_now) = {
        let s = state.read();
        (s.query.clone(), s.highlighted.clone(), s.open.value)
    };
    let groups: Vec<(String, Vec<CommandItem>)> = h::grouped(&items.read(), &query)
        .into_iter()
        .map(|(group, list)| (group.to_string(), list.into_iter().cloned().collect()))
        .collect();
    let activedescendant = highlighted.as_deref().map(|item| h::item_id(&root, item));

    rsx! {
        dialog {
            id: h::content_id(&root),
            "aria-label": label.clone(),
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": h::data_state(open_now),
            "data-overlay": "true",
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    e.prevent_default();
                    send(CommandPaletteEvent::Close);
                }
            },
            oncancel: move |e| {
                e.prevent_default();
                send(CommandPaletteEvent::Close);
            },
            onpointerdown: move |e| {
                // Stops the press focusing the closing <dialog>, which would undo focus return.
                e.prevent_default();
                send(CommandPaletteEvent::Close);
            },
            div { "data-scope": h::SCOPE, "data-part": "panel", onpointerdown: move |e| e.stop_propagation(),
                div { "data-scope": h::SCOPE, "data-part": "search",
                    Icon { name: IconName::Search, size: 18 }
                    input {
                        id: h::input_id(&root),
                        r#type: "text",
                        role: "combobox",
                        "aria-expanded": "true",
                        "aria-controls": h::list_id(&root),
                        "aria-autocomplete": "list",
                        "aria-activedescendant": activedescendant,
                        autocomplete: "off",
                        spellcheck: "false",
                        autofocus: true,
                        placeholder,
                        "data-scope": h::SCOPE,
                        "data-part": "input",
                        value: query,
                        oninput: move |e| send(CommandPaletteEvent::Input(e.value())),
                        onkeydown: move |e| {
                            if let Some(event) = h::keydown(&e.key().to_string()) {
                                e.prevent_default();
                                send(event);
                            }
                        },
                    }
                }
                div {
                    id: h::list_id(&root),
                    role: "listbox",
                    "aria-label": label,
                    "data-scope": h::SCOPE,
                    "data-part": "list",
                    if groups.is_empty() {
                        div { "data-scope": h::SCOPE, "data-part": "empty", "No matches" }
                    }
                    for (index, (group, list)) in groups.into_iter().enumerate() {
                        div {
                            key: "{group}",
                            role: "group",
                            "aria-labelledby": h::group_label_id(&root, index),
                            "data-scope": h::SCOPE,
                            "data-part": "group",
                            div {
                                id: h::group_label_id(&root, index),
                                "data-scope": h::SCOPE,
                                "data-part": "group-label",
                                {group}
                            }
                            for item in list {
                                div {
                                    key: "{item.id}",
                                    id: h::item_id(&root, &item.id),
                                    role: "option",
                                    "aria-selected": (highlighted.as_deref() == Some(item.id.as_str())).to_string(),
                                    "data-scope": h::SCOPE,
                                    "data-part": "item",
                                    "data-highlighted": (highlighted.as_deref() == Some(item.id.as_str())).then_some(""),
                                    onpointerdown: move |e| e.prevent_default(),
                                    onpointermove: {
                                        let id = item.id.clone();
                                        move |_| send(CommandPaletteEvent::Highlight(id.clone()))
                                    },
                                    onclick: {
                                        let id = item.id.clone();
                                        move |_| send(CommandPaletteEvent::Run(id.clone()))
                                    },
                                    if let Some(icon) = item.icon {
                                        span { "data-scope": h::SCOPE, "data-part": "item-icon",
                                            Icon { name: icon, size: 16 }
                                        }
                                    }
                                    span { "data-scope": h::SCOPE, "data-part": "item-label", {item.label} }
                                    if let Some(hint) = item.hint {
                                        kbd { "data-scope": h::SCOPE, "data-part": "item-hint", {hint} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
