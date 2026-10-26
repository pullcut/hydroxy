//! `<ContextMenu>` / `<ContextMenuTrigger>` / `<ContextMenuContent>` / `<ContextMenuItem>`.
//! A menu that opens at the pointer on right click, or at the focused element on
//! Shift+F10 or the `ContextMenu` key. Runs the menu machine; the content is a
//! native `popover="auto"`, so Esc, outside clicks and focus return come from
//! the browser.

use leptos::prelude::*;
use oxide_headless::context_menu::{self as h, Highlight, MenuCommand, MenuEvent, MenuInput, MenuMachine, MenuState};
use oxide_statechart::Machine;

use crate::focus::focus_id;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::context_menu::MenuEntry;

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<MenuState>,
    input: Memo<MenuInput>,
    point: RwSignal<(f64, f64)>,
    on_open_change: Option<Callback<bool>>,
    on_select: Option<Callback<String>>,
}

impl Ctx {
    fn send(self, event: MenuEvent) {
        // Events can still arrive after unmount (toggle on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| MenuMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        if self.state.with_untracked(|s| *s != step.state) {
            self.state.set(step.state);
        }
        for command in step.commands {
            match command {
                MenuCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
                MenuCommand::Select(value) => {
                    if let Some(cb) = self.on_select {
                        cb.run(value);
                    }
                }
                // `hidePopover()` already returns focus to where it was before the menu opened.
                MenuCommand::FocusTrigger => {}
            }
        }
    }

    /// Open at `point`, moving the menu there if it is already open.
    fn open_at(self, point: (f64, f64)) {
        if self.state.with_untracked(|s| s.open) {
            self.send(MenuEvent::Close { restore_focus: false });
        }
        self.point.set(point);
        self.send(MenuEvent::Open(Highlight::First));
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
    let input = Memo::new(move |_| MenuInput { items: items.get(), open: open.get(), default_open: false });
    let state = RwSignal::new(input.with_untracked(MenuMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| MenuMachine::sync(s, i)));
        if state.with_untracked(|s| *s != synced) {
            state.set(synced);
        }
    });

    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    // The popover and DOM focus follow the state, so controlled changes land too.
    Effect::new(move |_| {
        let (open, highlighted) = state.with(|s| (s.open, s.highlighted.clone()));
        let root = id.get_value();
        if open {
            show_popover(&h::content_id(&root));
            if let Some(value) = highlighted {
                focus_id(&h::item_id(&root, &value));
            }
        } else {
            hide_popover(&h::content_id(&root));
        }
    });

    provide_context(Ctx { id, state, input, point: RwSignal::new((0.0, 0.0)), on_open_change, on_select });
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
            data-state=move || h::data_state(ctx.state.with(|s| s.open))
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
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s)));
    let anchor = move || {
        let (x, y) = ctx.point.get();
        h::anchor_style(&ctx.id.get_value(), x, y)
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
            on:keydown=move |ev| {
                if let Some((event, prevent)) = h::content_keydown(&ev.key()) {
                    if prevent {
                        ev.prevent_default();
                    }
                    ctx.send(event);
                }
            }
            on:toggle=move |ev| {
                // The browser closed it (Esc fallback or outside click): catch the machine up.
                let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                if !showing {
                    ctx.send(MenuEvent::Close { restore_focus: false });
                }
            }
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
    let ctx = use_ctx();
    let attrs = {
        let value = value.clone();
        Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::item_attrs(&ctx.id.get_value(), s, i, &value))))
    };
    let (on_click, on_hover, on_focus) = (value.clone(), value.clone(), value);
    view! {
        <button
            type="button"
            id=move || attrs.get().id
            role="menuitem"
            tabindex="-1"
            disabled=move || attrs.get().disabled
            data-scope=h::SCOPE
            data-part="item"
            data-highlighted=move || attrs.get().highlighted.then_some("")
            on:click=move |_| ctx.send(MenuEvent::Select(on_click.clone()))
            on:pointermove=move |_| ctx.send(MenuEvent::Highlight(on_hover.clone()))
            on:focus=move |_| ctx.send(MenuEvent::Highlight(on_focus.clone()))
        >
            <span data-scope=h::SCOPE data-part="item-label">{children()}</span>
            {hint.map(|hint| view! { <span data-scope=h::SCOPE data-part="item-hint">{hint}</span> })}
        </button>
    }
}
