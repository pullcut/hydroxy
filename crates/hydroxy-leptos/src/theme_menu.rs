//! `<ThemeMenu>`: an icon button that opens a menu of light, dark and system.
//! The choice sets `data-theme` on `<html>` and is saved in `localStorage`.
//! Built from the menu machine; the items are radio items.

use hydroxy_headless::menu::{self as m, MenuCommand, MenuEvent, MenuInput, MenuMachine, MenuState};
use hydroxy_headless::theme_menu::{self as h};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, closure::Closure};

use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::theme_menu::{Align, Placement, Side, Theme};

fn system_dark() -> bool {
    window().match_media(h::DARK_QUERY).ok().flatten().is_some_and(|q| q.matches())
}

fn apply(theme: Theme) {
    if let Some(root) = document().document_element() {
        let _ = root.set_attribute("data-theme", theme.data_theme(system_dark()));
    }
}

fn saved() -> Option<Theme> {
    let storage = window().local_storage().ok().flatten()?;
    storage.get_item(h::STORAGE_KEY).ok().flatten().map(|v| Theme::parse(&v))
}

fn save(theme: Theme) {
    if let Ok(Some(storage)) = window().local_storage() {
        let _ = match theme.stored() {
            Some(value) => storage.set_item(h::STORAGE_KEY, value),
            None => storage.remove_item(h::STORAGE_KEY),
        };
    }
}

fn input() -> MenuInput {
    MenuInput { items: h::entries(), ..MenuInput::default() }
}

/// An icon button that opens the theme menu.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree")]
pub fn ThemeMenu(
    /// The choice before one is saved. Default system.
    #[prop(default = Theme::System)]
    default_value: Theme,
    /// Where the menu sits relative to the button. Default below, end-aligned.
    #[prop(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new choice when one is picked.
    #[prop(optional, into)]
    on_change: Option<Callback<Theme>>,
) -> impl IntoView {
    let theme = RwSignal::new(default_value);
    let state = RwSignal::new(MenuMachine::initial(&input()));
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));

    sync_document(theme);

    let send = move |event: MenuEvent| {
        let Some(step) = state.try_with_untracked(|s| MenuMachine::transition(s, event, &input())) else {
            return;
        };
        if let Some(next) = step.state
            && state.with_untracked(|s| *s != next)
        {
            state.set(next);
        }
        for command in step.commands {
            match command {
                MenuCommand::Select(value) => {
                    let choice = Theme::parse(&value);
                    theme.set(choice);
                    save(choice);
                    if let Some(cb) = on_change {
                        cb.run(choice);
                    }
                }
                MenuCommand::FocusTrigger => focus_id(&m::trigger_id(&id.get_value())),
                MenuCommand::OpenChange(_) => {}
            }
        }
    };

    // The popover and DOM focus follow the state.
    Effect::new(move |_| {
        let (open, highlighted) = state.with(|s| (s.open, s.highlighted.clone()));
        let root = id.get_value();
        if open {
            // Another control (the docs header toggle) may have changed the saved choice.
            if let Some(choice) = saved().filter(|c| theme.get_untracked() != *c) {
                theme.set(choice);
            }
            show_popover(&m::content_id(&root));
            if let Some(value) = highlighted {
                focus_id(&m::item_id(&root, &value));
            }
        } else {
            hide_popover(&m::content_id(&root));
        }
    });

    let trigger = Memo::new(move |_| state.with(|s| m::trigger_attrs(&id.get_value(), s)));
    let content = Memo::new(move |_| state.with(|s| m::content_attrs(&id.get_value(), s, placement)));
    view! {
        <button
            type="button"
            id=move || trigger.get().id
            aria-label="Theme"
            title="Theme"
            aria-haspopup="menu"
            aria-expanded=move || trigger.get().aria_expanded.to_string()
            aria-controls=move || trigger.get().aria_controls
            popovertarget=move || trigger.get().aria_controls
            style=move || trigger.get().style
            class=ICON_BUTTON_CLASS
            data-variant="ghost"
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || trigger.get().data_state
            on:click=move |ev| {
                // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                ev.prevent_default();
                send(MenuEvent::Toggle);
            }
            on:keydown=move |ev| {
                if let Some(event) = h::trigger_keydown(&ev.key()) {
                    ev.prevent_default();
                    send(event);
                }
            }
        >
            {move || view! { <Icon name=h::icon(theme.get()) size=18 /> }}
        </button>
        <div
            id=move || content.get().id
            role="menu"
            popover="auto"
            aria-labelledby=move || content.get().aria_labelledby
            style=move || content.get().style
            data-overlay=""
            data-side=move || content.get().data_side
            data-scope=m::SCOPE
            data-part="content"
            data-state=move || content.get().data_state
            on:keydown=move |ev| {
                if let Some((event, prevent)) = h::content_keydown(&ev.key()) {
                    if prevent {
                        ev.prevent_default();
                    }
                    send(event);
                }
            }
            on:toggle=move |ev| {
                // The browser closed it (Esc fallback or outside press): catch the machine up.
                let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                if !showing {
                    send(MenuEvent::Close { restore_focus: false });
                }
            }
        >
            {Theme::ALL.map(|choice| item(choice, id, state, theme, send))}
        </div>
    }
}

/// One radio item.
fn item(
    choice: Theme,
    id: StoredValue<String>,
    state: RwSignal<MenuState>,
    theme: RwSignal<Theme>,
    send: impl Fn(MenuEvent) + Copy + 'static,
) -> impl IntoView {
    let value = choice.as_str();
    let attrs = Memo::new(move |_| state.with(|s| h::item_attrs(&id.get_value(), s, &input(), value)));
    view! {
        <button
            type="button"
            id=move || attrs.get().id
            role="menuitemradio"
            aria-checked=move || (theme.get() == choice).to_string()
            tabindex="-1"
            data-scope=m::SCOPE
            data-part="item"
            data-highlighted=move || attrs.get().highlighted.then_some("")
            on:click=move |_| send(MenuEvent::Select(value.to_string()))
            on:pointermove=move |_| send(MenuEvent::Highlight(value.to_string()))
            on:focus=move |_| send(MenuEvent::Highlight(value.to_string()))
        >
            <Icon name=h::icon(choice) size=16 />
            <span data-scope=m::SCOPE data-part="item-label">{choice.label()}</span>
            <span data-scope=h::SCOPE data-part="check" aria-hidden="true">
                <Icon name=IconName::Check size=14 />
            </span>
        </button>
    }
}

/// On the client only: pick up the saved choice, apply it, and follow the system while it is chosen.
fn sync_document(theme: RwSignal<Theme>) {
    Effect::new(move |_| {
        if let Some(choice) = saved() {
            theme.set(choice);
        }
        if let Some(query) = window().match_media(h::DARK_QUERY).ok().flatten() {
            let follow = Closure::<dyn FnMut()>::new(move || {
                // The page may be gone by the time the system theme changes. A choice saved by
                // someone else (the docs header toggle) also wins over following the system.
                let following =
                    theme.try_get_untracked() == Some(Theme::System) && saved().is_none_or(|t| t == Theme::System);
                if following {
                    apply(Theme::System);
                }
            });
            query.set_onchange(Some(follow.as_ref().unchecked_ref()));
            // closure leaked once per mount (a few bytes); detached again on cleanup.
            follow.forget();
            let query = StoredValue::new_local(query);
            on_cleanup(move || {
                let _ = query.try_with_value(|q| q.set_onchange(None));
            });
        }
    });
    Effect::new(move |_| apply(theme.get()));
}
