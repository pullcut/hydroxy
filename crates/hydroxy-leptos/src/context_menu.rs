//! `<ContextMenu>` / `<ContextMenuTrigger>` / `<ContextMenuContent>` / `<ContextMenuItem>`.
//! A menu that opens at the pointer on right click, or at the focused element on
//! Shift+F10 or the `ContextMenu` key. Runs the menu machine; the content is a
//! native `popover="auto"`, so Esc, outside clicks and focus return come from
//! the browser.

use hydroxy_headless::context_menu::{self as h, Highlight, MenuEvent};
use hydroxy_headless::menu::DEFAULT_PLACEMENT;
use leptos::prelude::*;

use crate::menu::{self, Ctx as MenuCtx};

pub use hydroxy_headless::context_menu::MenuEntry;

/// The shared menu machine, plus the point the menu opens at.
#[derive(Clone, Copy)]
struct Ctx {
    menu: MenuCtx,
    point: RwSignal<(f64, f64)>,
}

impl Ctx {
    /// Open at `point`, moving the menu there if it is already open.
    fn open_at(self, point: (f64, f64)) {
        if self.menu.is_open() {
            self.menu.send(MenuEvent::Close { restore_focus: false });
        }
        self.point.set(point);
        self.menu.send(MenuEvent::Open(Highlight::First));
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("ContextMenu parts must be rendered inside <ContextMenu>")
}

/// Bottom-left of the focused element, where a keyboard-opened menu goes.
fn focused_point() -> (f64, f64) {
    document().active_element().map_or((0.0, 0.0), |el| {
        let rect = el.get_bounding_client_rect();
        (rect.left(), rect.bottom())
    })
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn ContextMenu(
    /// The items, in display order.
    #[prop(into)]
    items: Signal<Vec<MenuEntry>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Called when the menu opens or closes, or a change is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// Called with the activated item's value.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let menu = MenuCtx::new(h::SCOPE, items, open, false, DEFAULT_PLACEMENT, on_open_change, on_select);
    provide_context(Ctx { menu, point: RwSignal::new((0.0, 0.0)) });
    children()
}

/// The area that opens the menu on right click. Focusable, so Shift+F10 and the
/// `ContextMenu` key work from the keyboard.
#[component]
pub fn ContextMenuTrigger(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <div
            tabindex="0"
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || h::data_state(ctx.menu.open())
            on:contextmenu=move |ev| {
                ev.prevent_default();
                let client = (f64::from(ev.client_x()), f64::from(ev.client_y()));
                ctx.open_at(h::open_point(client, focused_point()));
            }
            on:keydown=move |ev| {
                if h::opens_from_key(&ev.key(), ev.shift_key()) {
                    ev.prevent_default();
                    ctx.open_at(focused_point());
                }
            }
        >
            {children()}
        </div>
    }
}

/// The floating list of items.
#[component]
pub fn ContextMenuContent(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let menu = ctx.menu;
    let attrs = Memo::new(move |_| menu.with_state(|s| h::content_attrs(&menu.root_id(), s)));
    let anchor = move || {
        let (x, y) = ctx.point.get();
        h::anchor_style(&menu.root_id(), x, y)
    };
    view! {
        <span aria-hidden="true" data-scope=h::SCOPE data-part="anchor" style=anchor></span>
        <div
            id=move || attrs.get().id
            role="menu"
            popover="auto"
            style=move || attrs.get().style
            data-overlay=""
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.get().data_state
            on:contextmenu=|ev| ev.prevent_default()
            on:keydown=move |ev| menu.content_keydown(&ev)
            on:toggle=move |ev| menu.toggled(&ev)
        >
            {children()}
        </div>
    }
}

/// One action. Children are the label; `hint` shows right-aligned (a shortcut, say).
#[component]
pub fn ContextMenuItem(
    /// The item this renders. Must match an entry in `items`.
    #[prop(into)]
    value: String,
    /// Right-aligned hint, such as a keyboard shortcut.
    #[prop(optional, into)]
    hint: Option<String>,
    children: Children,
) -> impl IntoView {
    menu::item(use_ctx().menu, h::SCOPE, value, hint, children)
}
