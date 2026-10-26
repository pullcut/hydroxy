//! `ContextMenu` / `ContextMenuTrigger` / `ContextMenuContent` / `ContextMenuItem`.
//! A menu that opens at the pointer on right click, or at the focused element on
//! Shift+F10 or the `ContextMenu` key. Runs the menu machine; the content is a
//! native `popover="auto"`, so Esc, outside clicks and focus return come from
//! the browser.

use dioxus::prelude::*;
use oxide_headless::context_menu::{self as h, Highlight, MenuCommand, MenuEvent, MenuInput, MenuMachine, MenuState};
use oxide_statechart::Machine;

use crate::focus::FocusMap;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::context_menu::MenuEntry;

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<MenuState>,
    input: Memo<MenuInput>,
    point: Signal<(f64, f64)>,
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
                // `hidePopover()` already returns focus to where it was before the menu opened.
                MenuCommand::FocusTrigger => {}
            }
        }
    }

    /// Open at `point`, moving the menu there if it is already open.
    fn open_at(mut self, point: (f64, f64)) {
        if self.state.peek().open {
            self.send(MenuEvent::Close { restore_focus: false });
        }
        self.point.set(point);
        self.send(MenuEvent::Open(Highlight::First));
    }

    /// Open at the focused element's bottom-left, for keyboard-opened menus.
    fn open_at_focus(self) {
        spawn(async move {
            let js = "const r = document.activeElement?.getBoundingClientRect(); return r ? [r.left, r.bottom] : [0, 0];";
            let point = document::eval(js).join::<(f64, f64)>().await.unwrap_or((0.0, 0.0));
            self.open_at(point);
        });
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("ContextMenu parts must be rendered inside ContextMenu")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn ContextMenu(
    /// The items, in display order.
    items: ReadSignal<Vec<MenuEntry>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Called when the menu opens or closes, or a change is requested.
    on_open_change: Option<EventHandler<bool>>,
    /// Called with the activated item's value.
    on_select: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let input = use_memo(move || MenuInput { items: items(), open: open(), default_open: false });
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
        point: Signal::new((0.0, 0.0)),
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

/// The area that opens the menu on right click. Focusable, so Shift+F10 and the
/// `ContextMenu` key work from the keyboard.
#[component]
pub fn ContextMenuTrigger(children: Element) -> Element {
    let ctx = use_ctx();
    rsx! {
        div {
            tabindex: "0",
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": h::data_state(ctx.state.read().open),
            oncontextmenu: move |e| {
                e.prevent_default();
                let p = e.client_coordinates();
                // Keyboard-fired events can report (0, 0): fall back to the focused element.
                if (p.x, p.y) == (0.0, 0.0) {
                    ctx.open_at_focus();
                } else {
                    ctx.open_at((p.x, p.y));
                }
            },
            onkeydown: move |e| {
                if h::opens_from_key(&e.key().to_string(), e.modifiers().shift()) {
                    e.prevent_default();
                    ctx.open_at_focus();
                }
            },
            {children}
        }
    }
}

/// The floating list of items.
#[component]
pub fn ContextMenuContent(children: Element) -> Element {
    let ctx = use_ctx();
    let root = ctx.id.read().clone();
    let a = h::content_attrs(&root, &ctx.state.read());
    let (x, y) = *ctx.point.read();
    let content = a.id.clone();
    rsx! {
        span {
            "aria-hidden": "true",
            "data-scope": h::SCOPE,
            "data-part": "anchor",
            style: h::anchor_style(&root, x, y),
        }
        div {
            id: a.id,
            role: "menu",
            "popover": "auto",
            style: a.style,
            "data-overlay": "",
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            oncontextmenu: move |e| e.prevent_default(),
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
            {children}
        }
    }
}

/// One action. Children are the label; `hint` shows right-aligned (a shortcut, say).
#[component]
pub fn ContextMenuItem(
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
