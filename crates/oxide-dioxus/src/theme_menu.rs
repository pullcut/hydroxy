//! `ThemeMenu`: an icon button that opens a menu of light, dark and system.
//! The choice sets `data-theme` on `<html>` and is saved in `localStorage`.
//! Built from the menu machine; the items are radio items. The page is reached
//! through `document::eval`, so it works on web and in a desktop webview.

use dioxus::prelude::*;
use oxide_headless::menu::{self as m, MenuCommand, MenuEvent, MenuInput, MenuMachine};
use oxide_headless::theme_menu::{self as h};
use oxide_statechart::Machine;

use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::theme_menu::{Align, Placement, Side, Theme};

/// `FocusMap` key for the trigger. Item values never start with a NUL.
const TRIGGER_KEY: &str = "\0trigger";

fn icon(theme: Theme) -> IconName {
    match theme {
        Theme::Light => IconName::Sun,
        Theme::Dark => IconName::Moon,
        Theme::System => IconName::Monitor,
        other if other.is_dark(false) => IconName::Moon,
        _ => IconName::Sun,
    }
}

fn input() -> MenuInput {
    MenuInput { items: h::entries(), ..MenuInput::default() }
}

/// Set `data-theme` for the choice. `save` also stores it (or clears the stored value for system).
fn apply_js(theme: Theme, save: bool) -> String {
    let store = match (save, theme.stored()) {
        (false, _) => String::new(),
        (true, Some(v)) => format!("localStorage.setItem({:?}, {v:?});", h::STORAGE_KEY),
        (true, None) => format!("localStorage.removeItem({:?});", h::STORAGE_KEY),
    };
    let dark = theme == Theme::System;
    let fixed = theme.data_theme(false);
    let value = if dark {
        format!("(matchMedia({:?}).matches ? 'dark' : 'light')", h::DARK_QUERY)
    } else {
        format!("{fixed:?}")
    };
    format!("try {{ {store} document.documentElement.dataset.theme = {value}; }} catch (_) {{}}")
}

/// An icon button that opens the theme menu.
#[component]
pub fn ThemeMenu(
    /// The choice before one is saved. Default system.
    #[props(default = Theme::System)]
    default_value: Theme,
    /// Where the menu sits relative to the button. Default below, end-aligned.
    #[props(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new choice when one is picked.
    on_change: Option<EventHandler<Theme>>,
) -> Element {
    let mut theme = use_signal(|| default_value);
    let mut state = use_signal(|| MenuMachine::initial(&input()));
    let id = use_hook(|| CopyValue::new(oxide_headless::next_id(h::SCOPE)));
    let focus = use_hook(FocusMap::new);

    // Pick up the saved choice once, then apply the current one and follow the system while it is chosen.
    use_effect(move || {
        spawn(async move {
            let js = format!("return localStorage.getItem({:?}) ?? '';", h::STORAGE_KEY);
            if let Ok(saved) = document::eval(&js).join::<String>().await
                && !saved.is_empty()
            {
                theme.set(Theme::parse(&saved));
            }
        });
        let js = format!(
            "matchMedia({q:?}).onchange = () => {{ if (!localStorage.getItem({k:?})) {{ document.documentElement.dataset.theme = matchMedia({q:?}).matches ? 'dark' : 'light'; }} }};",
            q = h::DARK_QUERY,
            k = h::STORAGE_KEY
        );
        let _ = document::eval(&js);
    });
    use_effect(move || {
        let _ = document::eval(&apply_js(theme(), false));
    });

    let mut send = move |event: MenuEvent| {
        let step = MenuMachine::transition(&state.peek(), event, &input());
        if let Some(next) = step.state
            && *state.peek() != next
        {
            state.set(next);
        }
        for command in step.commands {
            match command {
                MenuCommand::Select(value) => {
                    let choice = Theme::parse(&value);
                    theme.set(choice);
                    let _ = document::eval(&apply_js(choice, true));
                    if let Some(handler) = on_change {
                        handler.call(choice);
                    }
                }
                MenuCommand::FocusTrigger => focus.focus(TRIGGER_KEY),
                MenuCommand::OpenChange(_) => {}
            }
        }
    };

    // The popover and DOM focus follow the state.
    use_effect(move || {
        let (open, highlighted) = {
            let s = state.read();
            (s.open, s.highlighted.clone())
        };
        let content = m::content_id(&id.peek());
        if open {
            show_popover(&content);
            if let Some(value) = highlighted {
                focus.focus(&value);
            }
        } else {
            hide_popover(&content);
        }
    });

    let root = id.read().clone();
    let trigger = m::trigger_attrs(&root, &state.read());
    let content = m::content_attrs(&root, &state.read(), placement);
    let content_el = content.id.clone();
    rsx! {
        button {
            r#type: "button",
            id: trigger.id,
            "aria-label": "Theme",
            title: "Theme",
            "aria-haspopup": "menu",
            "aria-expanded": trigger.aria_expanded.to_string(),
            "aria-controls": trigger.aria_controls.clone(),
            "popovertarget": trigger.aria_controls,
            style: trigger.style,
            class: ICON_BUTTON_CLASS,
            "data-variant": "ghost",
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": trigger.data_state,
            onmounted: move |e| focus.register(TRIGGER_KEY.to_string(), e.data()),
            onclick: move |e| {
                // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                e.prevent_default();
                send(MenuEvent::Toggle);
            },
            onkeydown: move |e| {
                if let Some(event) = h::trigger_keydown(&e.key().to_string()) {
                    e.prevent_default();
                    send(event);
                }
            },
            Icon { name: icon(theme()), size: 18 }
        }
        div {
            id: content.id,
            role: "menu",
            "popover": "auto",
            "aria-labelledby": content.aria_labelledby,
            style: content.style,
            "data-overlay": "",
            "data-side": content.data_side,
            "data-scope": m::SCOPE,
            "data-part": "content",
            "data-state": content.data_state,
            onkeydown: move |e| {
                if let Some((event, prevent)) = h::content_keydown(&e.key().to_string()) {
                    if prevent {
                        e.prevent_default();
                    }
                    send(event);
                }
            },
            ontoggle: move |_| {
                // The browser may have closed it (outside press): ask, then catch the machine up.
                let js = format!("return document.getElementById({content_el:?})?.matches(':popover-open') ?? false");
                spawn(async move {
                    if let Ok(false) = document::eval(&js).join::<bool>().await {
                        send(MenuEvent::Close { restore_focus: false });
                    }
                });
            },
            for choice in Theme::ALL {
                {
                    let value = choice.as_str();
                    let a = h::item_attrs(&root, &state.read(), &input(), value);
                    rsx! {
                        button {
                            r#type: "button",
                            id: a.id,
                            role: "menuitemradio",
                            "aria-checked": (theme() == choice).to_string(),
                            tabindex: "-1",
                            "data-scope": m::SCOPE,
                            "data-part": "item",
                            "data-highlighted": a.highlighted.then_some(""),
                            onmounted: move |e| focus.register(value.to_string(), e.data()),
                            onclick: move |_| send(MenuEvent::Select(value.to_string())),
                            onpointermove: move |_| send(MenuEvent::Highlight(value.to_string())),
                            onfocus: move |_| send(MenuEvent::Highlight(value.to_string())),
                            Icon { name: icon(choice), size: 16 }
                            span { "data-scope": m::SCOPE, "data-part": "item-label", "{choice.label()}" }
                            span { "data-scope": h::SCOPE, "data-part": "check", "aria-hidden": "true",
                                Icon { name: IconName::Check, size: 14 }
                            }
                        }
                    }
                }
            }
        }
    }
}
