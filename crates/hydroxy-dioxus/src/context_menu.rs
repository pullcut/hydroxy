//! `ContextMenu` / `ContextMenuTrigger` / `ContextMenuContent` / `ContextMenuItem`.
//! A menu that opens at the pointer on right click, or at the focused element on
//! Shift+F10 or the `ContextMenu` key. Runs the menu machine; the content is a
//! native `popover="auto"`, so Esc, outside clicks and focus return come from
//! the browser.
//!
//! Parts panic if rendered outside their root `ContextMenu`.

use dioxus::prelude::*;
use hydroxy_headless::context_menu::{self as h, Highlight, MenuEvent};

use crate::menu::{self, Ctx, MenuPanel, MenuRow, use_menu};
use crate::modifiers::mods;

pub use hydroxy_headless::context_menu::MenuEntry;

/// The shared menu plus the point the content anchors to.
#[derive(Clone, Copy)]
struct Opener {
    menu: Ctx,
    point: Signal<(f64, f64)>,
}

impl Opener {
    /// Open at `point`, moving the menu there if it is already open.
    fn open_at(mut self, point: (f64, f64)) {
        if self.menu.state.peek().open {
            self.menu.send(MenuEvent::Close { restore_focus: false });
        }
        self.point.set(point);
        self.menu.send(MenuEvent::Open(Highlight::First));
    }

    /// Open at the focused element's bottom-left, for keyboard-opened menus.
    fn open_at_focus(self) {
        spawn(async move {
            let js =
                "const r = document.activeElement?.getBoundingClientRect(); return r ? [r.left, r.bottom] : [0, 0];";
            let point = document::eval(js).join::<(f64, f64)>().await.unwrap_or((0.0, 0.0));
            self.open_at(point);
        });
    }
}

fn use_opener() -> Opener {
    try_use_context::<Opener>().expect("ContextMenu parts must be rendered inside ContextMenu")
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
    let menu = use_menu(h::SCOPE, items, open, false, menu::Placement::default(), on_open_change, on_select);
    use_context_provider(|| Opener { menu, point: Signal::new((0.0, 0.0)) });

    rsx! { {children} }
}

/// The area that opens the menu on right click. Focusable, so Shift+F10 and the
/// `ContextMenu` key work from the keyboard.
#[component]
pub fn ContextMenuTrigger(children: Element) -> Element {
    let ctx = use_opener();
    rsx! {
        div {
            tabindex: "0",
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": h::data_state(ctx.menu.state.read().open),
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
                if h::opens_from_key(&e.key().to_string(), mods(e.modifiers())) {
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
    let opener = use_opener();
    let ctx = opener.menu;
    let root = ctx.id.read().clone();
    let a = h::content_attrs(&root, &ctx.state.read());
    let (x, y) = *opener.point.read();
    rsx! {
        span {
            "aria-hidden": "true",
            "data-scope": h::SCOPE,
            "data-part": "anchor",
            style: h::anchor_style(&root, x, y),
        }
        MenuPanel {
            scope: h::SCOPE,
            id: a.id,
            style: a.style,
            data_state: a.data_state,
            suppress_context_menu: true,
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
    rsx! {
        MenuRow { scope: h::SCOPE, value, hint, {children} }
    }
}
