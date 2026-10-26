//! `Menu` / `MenuTrigger` / `MenuContent` / `MenuItem`.
//! Parts read the machine from context and render the connector's attributes.
//! The content is a native `popover="auto"`: top layer, Esc and outside-click
//! dismissal come from the browser; CSS anchor positioning places it.

use dioxus::prelude::*;
use oxide_headless::menu::{self as h, MenuCommand, MenuEvent, MenuInput, MenuMachine, MenuState};
use oxide_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};

use crate::focus::FocusMap;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::menu::MenuEntry;
pub use oxide_headless::overlay::{Align, Placement, Side};

/// `FocusMap` key for the trigger. Item values never start with a NUL.
const TRIGGER_KEY: &str = "\0trigger";

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<MenuState>,
    input: Memo<MenuInput>,
    placement: Placement,
    focus: FocusMap,
    on_open_change: Option<EventHandler<bool>>,
    on_select: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(mut self, event: MenuEvent) {
        let step = MenuMachine::transition(&self.state.peek(), event, &self.input.peek());
        if *self.state.peek() != step.state {
            self.state.set(step.state);
        }
        for command in step.commands {
            match command {
                MenuCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
                MenuCommand::Select(value) => {
                    if let Some(handler) = self.on_select {
                        handler.call(value);
                    }
                }
                MenuCommand::FocusTrigger => self.focus.focus(TRIGGER_KEY),
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Menu parts must be rendered inside Menu")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Menu(
    /// The items, in display order.
    items: ReadSignal<Vec<MenuEntry>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Where the menu sits relative to the trigger. Default below, start-aligned.
    #[props(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called when the menu opens or closes, or a change is requested.
    on_open_change: Option<EventHandler<bool>>,
    /// Called with the activated item's value.
    on_select: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let input = use_memo(move || MenuInput { items: items(), open: open(), default_open });
    let mut state = use_signal(|| MenuMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = MenuMachine::sync(&state.peek(), &input.read());
        if *state.peek() != synced {
            state.set(synced);
        }
    });

    let ctx = use_context_provider(|| Ctx {
        id: CopyValue::new(oxide_headless::next_id(h::SCOPE)),
        state,
        input,
        placement,
        focus: FocusMap::new(),
        on_open_change,
        on_select,
    });

    // The popover and DOM focus follow the state, so controlled changes land too.
    use_effect(move || {
        let (open, highlighted) = {
            let s = state.read();
            (s.open, s.highlighted.clone())
        };
        let content = h::content_id(&ctx.id.peek());
        if open {
            show_popover(&content);
            if let Some(value) = highlighted {
                ctx.focus.focus(&value);
            }
        } else {
            hide_popover(&content);
        }
    });

    rsx! { {children} }
}

/// The button that opens the menu.
#[component]
pub fn MenuTrigger(
    /// Button look. Default secondary.
    variant: Option<ButtonVariant>,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::trigger_attrs(&ctx.id.read(), &ctx.state.read());
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            "aria-haspopup": "menu",
            "aria-expanded": a.aria_expanded.to_string(),
            "aria-controls": a.aria_controls.clone(),
            "popovertarget": a.aria_controls,
            style: a.style,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": a.data_state,
            class: BUTTON_CLASS,
            "data-variant": variant.unwrap_or(ButtonVariant::Secondary).as_str(),
            onmounted: move |e| ctx.focus.register(TRIGGER_KEY.to_string(), e.data()),
            onclick: move |e| {
                // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                e.prevent_default();
                ctx.send(MenuEvent::Toggle);
            },
            onkeydown: move |e| {
                if let Some(event) = h::trigger_keydown(&e.key().to_string()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            {children}
        }
    }
}

/// The floating list of items. An optional `label` shows as a small heading.
#[component]
pub fn MenuContent(
    /// Small uppercase heading above the items.
    label: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read(), ctx.placement);
    let content = a.id.clone();
    rsx! {
        div {
            id: a.id,
            role: "menu",
            "popover": "auto",
            "aria-labelledby": a.aria_labelledby,
            style: a.style,
            "data-overlay": "",
            "data-side": a.data_side,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            onkeydown: move |e| {
                if let Some((event, prevent)) = h::content_keydown(&e.key().to_string()) {
                    if prevent {
                        e.prevent_default();
                    }
                    ctx.send(event);
                }
            },
            ontoggle: move |_| {
                // The browser may have closed it (outside click): ask, then catch the machine up.
                let js = format!("return document.getElementById({content:?})?.matches(':popover-open') ?? false");
                spawn(async move {
                    if let Ok(false) = document::eval(&js).join::<bool>().await {
                        ctx.send(MenuEvent::Close { restore_focus: false });
                    }
                });
            },
            if let Some(label) = label {
                div { "data-scope": h::SCOPE, "data-part": "label", "aria-hidden": "true", "{label}" }
            }
            {children}
        }
    }
}

/// One action. Children are the label; `hint` shows right-aligned (a shortcut, say).
#[component]
pub fn MenuItem(
    /// The item this renders. Must match an entry in `items`.
    value: String,
    /// Right-aligned hint, such as a keyboard shortcut.
    hint: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::item_attrs(&ctx.id.read(), &ctx.state.read(), &ctx.input.read(), &value);
    let (mounted, clicked, hovered, focused) = (value.clone(), value.clone(), value.clone(), value);
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            role: "menuitem",
            tabindex: "-1",
            disabled: a.disabled,
            "data-scope": h::SCOPE,
            "data-part": "item",
            "data-highlighted": a.highlighted.then_some(""),
            onmounted: move |e| ctx.focus.register(mounted.clone(), e.data()),
            onclick: move |_| ctx.send(MenuEvent::Select(clicked.clone())),
            onpointermove: move |_| ctx.send(MenuEvent::Highlight(hovered.clone())),
            onfocus: move |_| ctx.send(MenuEvent::Highlight(focused.clone())),
            span { "data-scope": h::SCOPE, "data-part": "item-label", {children} }
            if let Some(hint) = hint {
                span { "data-scope": h::SCOPE, "data-part": "item-hint", "{hint}" }
            }
        }
    }
}
