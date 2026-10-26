//! `Menubar` / `MenubarMenu` / `MenubarItem`.
//! The bar owns which menu is open; each menu is a [`oxide_headless::menu`]
//! instance driven open or closed by the bar. Menu content is a native
//! `popover="auto"`, so Esc and outside presses are the browser's.

use dioxus::prelude::*;
use oxide_headless::menu::{self as m, MenuCommand, MenuEvent, MenuInput, MenuMachine, MenuState};
use oxide_headless::menubar::{self as h, MenubarCommand, MenubarEvent, MenubarInput, MenubarMachine, MenubarState};
use oxide_statechart::Machine;

use crate::focus::FocusMap;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::menu::MenuEntry;
pub use oxide_headless::menubar::MenubarEntry;

#[derive(Clone, Copy)]
struct BarCtx {
    id: CopyValue<String>,
    state: Signal<MenubarState>,
    input: Memo<MenubarInput>,
    /// Triggers, by menu value.
    focus: FocusMap,
    on_open_change: Option<EventHandler<Option<String>>>,
}

impl BarCtx {
    fn send(mut self, event: MenubarEvent) {
        let step = MenubarMachine::transition(&self.state.peek(), event, &self.input.peek());
        if *self.state.peek() != step.state {
            self.state.set(step.state);
        }
        for command in step.commands {
            match command {
                MenubarCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
                MenubarCommand::FocusTrigger(value) => self.focus.focus(&value),
            }
        }
    }
}

#[derive(Clone, Copy)]
struct MenuCtx {
    bar: BarCtx,
    root: CopyValue<String>,
    value: CopyValue<String>,
    state: Signal<MenuState>,
    input: Memo<MenuInput>,
    /// Items, by item value.
    focus: FocusMap,
    on_select: Option<EventHandler<String>>,
}

impl MenuCtx {
    fn send(mut self, event: MenuEvent) {
        let step = MenuMachine::transition(&self.state.peek(), event, &self.input.peek());
        if *self.state.peek() != step.state {
            self.state.set(step.state);
        }
        for command in step.commands {
            match command {
                // The menu closed itself: the bar decides, and the popover follows the bar.
                MenuCommand::OpenChange(false) => self.bar.send(MenubarEvent::Dismiss(self.value.peek().clone())),
                MenuCommand::OpenChange(true) => {}
                MenuCommand::Select(value) => {
                    if let Some(handler) = self.on_select {
                        handler.call(value);
                    }
                }
                MenuCommand::FocusTrigger => self.bar.focus.focus(&self.value.peek()),
            }
        }
    }
}

fn use_bar() -> BarCtx {
    try_use_context::<BarCtx>().expect("Menubar parts must be rendered inside Menubar")
}

fn use_menu() -> MenuCtx {
    try_use_context::<MenuCtx>().expect("MenubarItem must be rendered inside MenubarMenu")
}

/// Root: a horizontal bar of menus. Owns which menu is open.
#[component]
pub fn Menubar(
    /// The menus, in display order. Must match the `value`s of the `MenubarMenu`s.
    menus: ReadSignal<Vec<MenubarEntry>>,
    /// Accessible name for the bar.
    #[props(default = "Menu bar".to_string())]
    label: String,
    /// Controlled open menu (`Some(None)` is none open). Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<Option<String>>>,
    /// Initial open menu when uncontrolled.
    default_open: Option<String>,
    /// Called with the open menu's value, or `None`, whenever it changes or is requested.
    on_open_change: Option<EventHandler<Option<String>>>,
    children: Element,
) -> Element {
    let input = use_memo(move || MenubarInput { menus: menus(), open: open(), default_open: default_open.clone() });
    let mut state = use_signal(|| MenubarMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = MenubarMachine::sync(&state.peek(), &input.read());
        if *state.peek() != synced {
            state.set(synced);
        }
    });
    let ctx = use_context_provider(|| BarCtx {
        id: CopyValue::new(oxide_headless::next_id(h::SCOPE)),
        state,
        input,
        focus: FocusMap::new(),
        on_open_change,
    });
    rsx! {
        div {
            role: "menubar",
            "aria-label": "{label}",
            "aria-orientation": "horizontal",
            "data-scope": h::SCOPE,
            "data-part": "root",
            onkeydown: move |e| {
                if let Some(event) = h::root_keydown(&e.key().to_string()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            {children}
        }
    }
}

/// One menu in the bar: its trigger and its popover. Children are `MenubarItem`s.
#[component]
pub fn MenubarMenu(
    /// Unique key. Must match an entry in the bar's `menus`.
    value: String,
    /// Text of the trigger.
    label: String,
    /// The items, in display order.
    items: ReadSignal<Vec<MenuEntry>>,
    /// Called with the activated item's value.
    on_select: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let bar = use_bar();
    let root = CopyValue::new(h::menu_root(&bar.id.peek(), &value));
    let value_cell = CopyValue::new(value);
    let is_open = use_memo(move || bar.state.read().open.as_deref() == Some(value_cell.peek().as_str()));
    let input = use_memo(move || MenuInput { items: items(), open: Some(is_open()), default_open: false });
    let mut state = use_signal(|| MenuMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = MenuMachine::sync(&state.peek(), &input.read());
        if *state.peek() != synced {
            state.set(synced);
        }
    });
    let ctx = use_context_provider(|| MenuCtx {
        bar,
        root,
        value: value_cell,
        state,
        input,
        focus: FocusMap::new(),
        on_select,
    });
    // The popover and DOM focus follow the state. A menu opened by the bar has no
    // highlight yet, so `Home` picks the first item.
    use_effect(move || {
        let (open, highlighted) = {
            let s = state.read();
            (s.open, s.highlighted.clone())
        };
        let content = m::content_id(&root.peek());
        if open {
            show_popover(&content);
            match highlighted {
                Some(item) => ctx.focus.focus(&item),
                None => ctx.send(MenuEvent::Home),
            }
        } else {
            hide_popover(&content);
            if highlighted.is_some() {
                state.write().highlighted = None;
            }
        }
    });

    let trigger = m::trigger_attrs(&root.read(), &state.read());
    let content = m::content_attrs(&root.read(), &state.read(), m::DEFAULT_PLACEMENT);
    let bar_attrs = h::trigger_attrs(&bar.state.read(), &bar.input.read(), &value_cell.read());
    let content_el = content.id.clone();
    rsx! {
        button {
            r#type: "button",
            id: trigger.id,
            role: "menuitem",
            "aria-haspopup": "menu",
            "aria-expanded": trigger.aria_expanded.to_string(),
            "aria-controls": trigger.aria_controls.clone(),
            "popovertarget": trigger.aria_controls,
            style: trigger.style,
            tabindex: bar_attrs.tabindex,
            disabled: bar_attrs.disabled,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": bar_attrs.data_state,
            onmounted: move |e| bar.focus.register(value_cell.peek().clone(), e.data()),
            onclick: move |e| {
                // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                e.prevent_default();
                bar.send(MenubarEvent::Toggle(value_cell.peek().clone()));
            },
            onkeydown: move |e| {
                if let Some(event) = h::trigger_keydown(&e.key().to_string(), &value_cell.peek()) {
                    e.prevent_default();
                    bar.send(event);
                }
            },
            onpointerenter: move |_| bar.send(MenubarEvent::Hover(value_cell.peek().clone())),
            onfocus: move |_| bar.send(MenubarEvent::Focus(value_cell.peek().clone())),
            "{label}"
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
                if let Some((event, prevent)) = m::content_keydown(&e.key().to_string()) {
                    if prevent {
                        e.prevent_default();
                    }
                    ctx.send(event);
                }
            },
            ontoggle: move |_| {
                // The browser may have closed it (outside press, or another menu opened): ask, then catch up.
                let js = format!("return document.getElementById({content_el:?})?.matches(':popover-open') ?? false");
                spawn(async move {
                    if let Ok(false) = document::eval(&js).join::<bool>().await {
                        ctx.send(MenuEvent::Close { restore_focus: false });
                    }
                });
            },
            {children}
        }
    }
}

/// One action in a menu. Children are the label; `hint` shows right-aligned.
#[component]
pub fn MenubarItem(
    /// The item this renders. Must match an entry in the menu's `items`.
    value: String,
    /// Right-aligned hint, such as a keyboard shortcut.
    hint: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_menu();
    let a = m::item_attrs(&ctx.root.read(), &ctx.state.read(), &ctx.input.read(), &value);
    let (mounted, clicked, hovered, focused) = (value.clone(), value.clone(), value.clone(), value);
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            role: "menuitem",
            tabindex: "-1",
            disabled: a.disabled,
            "data-scope": m::SCOPE,
            "data-part": "item",
            "data-highlighted": a.highlighted.then_some(""),
            onmounted: move |e| ctx.focus.register(mounted.clone(), e.data()),
            onclick: move |_| ctx.send(MenuEvent::Select(clicked.clone())),
            onpointermove: move |_| ctx.send(MenuEvent::Highlight(hovered.clone())),
            onfocus: move |_| ctx.send(MenuEvent::Highlight(focused.clone())),
            span { "data-scope": m::SCOPE, "data-part": "item-label", {children} }
            if let Some(hint) = hint {
                span { "data-scope": m::SCOPE, "data-part": "item-hint", "{hint}" }
            }
        }
    }
}
