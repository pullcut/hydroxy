//! `<CommandPalette>`: a modal search over commands, on a native `<dialog>`
//! opened with `showModal()`. Focus stays in the search input; the highlight is
//! announced with `aria-activedescendant`.

use hydroxy_headless::command_palette::{
    self as h, CommandPaletteCommand, CommandPaletteEvent, CommandPaletteInput, CommandPaletteMachine,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::combobox::scroll_into_list;
use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::overlay::{close_dialog, show_modal};

pub use hydroxy_headless::command_palette::CommandItem;

/// A command palette. Open it from a button or a shortcut; Enter runs the
/// highlighted command and closes, Esc or a press outside closes.
#[component]
pub fn CommandPalette(
    /// The commands. Shown grouped, groups in first-appearance order.
    #[prop(into)]
    items: Signal<Vec<CommandItem>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Search placeholder.
    #[prop(default = "Search or run a command…".into(), into)]
    placeholder: String,
    /// Accessible name for the dialog and list.
    #[prop(default = "Command palette".into(), into)]
    label: String,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// Called with the id of the command that ran.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
) -> impl IntoView {
    let input = Memo::new(move |_| CommandPaletteInput {
        entries: items.with(|i| h::entries(i)),
        open: open.get(),
        default_open,
    });
    let state = RwSignal::new(input.with_untracked(CommandPaletteMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CommandPaletteMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let root = crate::id::next_id(h::SCOPE);
    let id = StoredValue::new(root.clone());
    // The native dialog follows the state; keep the highlighted command in view.
    Effect::new(move |_| {
        let (open, highlighted) = state.with(|s| (s.open.value, s.highlighted.clone()));
        let root = id.get_value();
        if open {
            show_modal(&h::content_id(&root));
            if let Some(item) = highlighted {
                scroll_into_list(&h::list_id(&root), &h::item_id(&root, &item));
            }
        } else {
            close_dialog(&h::content_id(&root));
        }
    });

    let send = move |event: CommandPaletteEvent| {
        for command in drive::<CommandPaletteMachine, _>(state, input, event) {
            match command {
                CommandPaletteCommand::OpenChange(open) => {
                    if let Some(cb) = on_open_change {
                        cb.run(open);
                    }
                }
                CommandPaletteCommand::Select(item) => {
                    if let Some(cb) = on_select {
                        cb.run(item);
                    }
                }
            }
        }
    };

    // Keyed groups and rows: typing updates only what changed, not the whole list.
    let groups = Memo::new(move |_| {
        let query = state.with(|s| s.query.clone());
        items.with(|all| {
            h::grouped(all, &query)
                .into_iter()
                .map(|(group, list)| (group.to_string(), list.into_iter().cloned().collect::<Vec<CommandItem>>()))
                .collect::<Vec<_>>()
        })
    });
    let item_view = move |item: CommandItem| {
        let on = {
            let id = item.id.clone();
            Memo::new(move |_| state.with(|s| s.highlighted.as_deref() == Some(id.as_str())))
        };
        let (hovered, clicked) = (item.id.clone(), item.id.clone());
        view! {
            <div
                id=h::item_id(&id.get_value(), &item.id)
                role="option"
                aria-selected=move || on.get().to_string()
                data-scope=h::SCOPE
                data-part="item"
                data-highlighted=move || on.get().then_some("")
                on:pointerdown=|ev| ev.prevent_default()
                on:pointermove=move |_| {
                    if !on.get_untracked() {
                        send(CommandPaletteEvent::Highlight(hovered.clone()));
                    }
                }
                on:click=move |_| send(CommandPaletteEvent::Run(clicked.clone()))
            >
                {item
                    .icon
                    .map(|icon| {
                        view! {
                            <span data-scope=h::SCOPE data-part="item-icon">
                                <Icon name=icon size=16 />
                            </span>
                        }
                    })}
                <span data-scope=h::SCOPE data-part="item-label">{item.label}</span>
                {item.hint.map(|hint| view! { <kbd data-scope=h::SCOPE data-part="item-hint">{hint}</kbd> })}
            </div>
        }
    };
    let group_view = move |(index, group): (usize, String)| {
        let heading = h::group_label_id(&id.get_value(), index);
        let labelled_by = heading.clone();
        let label = group.clone();
        let rows = move || {
            groups.with(|g| g.iter().find(|(name, _)| *name == group).map(|(_, l)| l.clone()).unwrap_or_default())
        };
        view! {
            <div role="group" aria-labelledby=labelled_by data-scope=h::SCOPE data-part="group">
                <div id=heading data-scope=h::SCOPE data-part="group-label">{label}</div>
                <For each=rows key=|item| (item.id.clone(), item.label.clone()) children=item_view />
            </div>
        }
    };
    let results = view! {
        <Show
            when=move || groups.with(|g| !g.is_empty())
            fallback=|| view! { <div data-scope=h::SCOPE data-part="empty">"No matches"</div> }
        >
            <For
                each=move || groups.with(|g| g.iter().enumerate().map(|(i, (name, _))| (i, name.clone())).collect::<Vec<_>>())
                key=|pair| pair.clone()
                children=group_view
            />
        </Show>
    };

    let list_label = label.clone();
    view! {
        <dialog
            id=h::content_id(&root)
            aria-label=label
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || h::data_state(state.with(|s| s.open.value))
            data-overlay="true"
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    ev.prevent_default();
                    send(CommandPaletteEvent::Close);
                }
            }
            on:cancel=move |ev: leptos::ev::Event| {
                ev.prevent_default();
                send(CommandPaletteEvent::Close);
            }
            on:pointerdown=move |ev| {
                // Stops the press focusing the closing <dialog>, which would undo focus return.
                ev.prevent_default();
                send(CommandPaletteEvent::Close);
            }
        >
            <div data-scope=h::SCOPE data-part="panel" on:pointerdown=|ev| ev.stop_propagation()>
                <div data-scope=h::SCOPE data-part="search">
                    <Icon name=IconName::Search size=18 />
                    <input
                        id=h::input_id(&root)
                        type="text"
                        role="combobox"
                        aria-expanded="true"
                        aria-controls=h::list_id(&root)
                        aria-autocomplete="list"
                        aria-activedescendant=move || state.with(|s| h::activedescendant(&id.get_value(), s))
                        autocomplete="off"
                        spellcheck="false"
                        autofocus=true
                        placeholder=placeholder
                        data-scope=h::SCOPE
                        data-part="input"
                        prop:value=move || state.with(|s| s.query.clone())
                        on:input=move |ev| send(CommandPaletteEvent::Input(event_target_value(&ev)))
                        on:keydown=move |ev| {
                            if let Some(event) = h::keydown(&ev.key()) {
                                ev.prevent_default();
                                send(event);
                            }
                        }
                    />
                </div>
                <div id=h::list_id(&root) role="listbox" aria-label=list_label data-scope=h::SCOPE data-part="list">
                    {results}
                </div>
            </div>
        </dialog>
    }
}
