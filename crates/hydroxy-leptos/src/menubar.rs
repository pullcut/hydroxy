//! `<Menubar>` / `<MenubarMenu>` / `<MenubarItem>`.
//! The bar owns which menu is open; each menu is a [`hydroxy_headless::menu`]
//! instance driven open or closed by the bar. Menu content is a native
//! `popover="auto"`, so Esc and outside presses are the browser's.
//!
//! Parts panic if rendered outside their root `Menubar`; `MenubarItem` also needs a
//! `MenubarMenu`.

use crate::drive::drive;
use hydroxy_headless::menu::{self as m, MenuCommand, MenuEvent, MenuInput, MenuMachine, MenuState};
use hydroxy_headless::menubar::{self as h, MenubarCommand, MenubarEvent, MenubarInput, MenubarMachine, MenubarState};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::focus::focus_id;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::menu::MenuEntry;
pub use hydroxy_headless::menubar::MenubarEntry;

#[derive(Clone, Copy)]
struct BarCtx {
    id: StoredValue<String>,
    state: RwSignal<MenubarState>,
    input: Memo<MenubarInput>,
    on_open_change: Option<Callback<Option<String>>>,
}

impl BarCtx {
    fn send(self, event: MenubarEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        for command in drive::<MenubarMachine, _>(self.state, self.input, event) {
            match command {
                MenubarCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
                MenubarCommand::FocusTrigger(value) => {
                    focus_id(&m::trigger_id(&h::menu_root(&self.id.get_value(), &value)));
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct MenuCtx {
    bar: BarCtx,
    root: StoredValue<String>,
    value: StoredValue<String>,
    state: RwSignal<MenuState>,
    input: Memo<MenuInput>,
    on_select: Option<Callback<String>>,
}

impl MenuCtx {
    fn send(self, event: MenuEvent) {
        for command in drive::<MenuMachine, _>(self.state, self.input, event) {
            match command {
                // The menu closed itself: the bar decides, and the popover follows the bar.
                MenuCommand::OpenChange(false) => self.bar.send(MenubarEvent::Dismiss(self.value.get_value())),
                MenuCommand::OpenChange(true) => {}
                MenuCommand::Select(value) => {
                    if let Some(cb) = self.on_select {
                        cb.run(value);
                    }
                }
                MenuCommand::FocusTrigger => focus_id(&m::trigger_id(&self.root.get_value())),
            }
        }
    }
}

fn use_bar() -> BarCtx {
    use_context::<BarCtx>().expect("Menubar parts must be rendered inside <Menubar>")
}

fn use_menu() -> MenuCtx {
    use_context::<MenuCtx>().expect("MenubarItem must be rendered inside <MenubarMenu>")
}

/// Root: a horizontal bar of menus. Owns which menu is open.
#[component]
pub fn Menubar(
    /// The menus, in display order. Must match the `value`s of the `MenubarMenu`s.
    #[prop(into)]
    menus: Signal<Vec<MenubarEntry>>,
    /// Accessible name for the bar.
    #[prop(default = "Menu bar".into(), into)]
    label: String,
    /// Controlled open menu (`Some(None)` is none open). Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<Option<String>>,
    /// Initial open menu when uncontrolled.
    #[prop(optional, into)]
    default_open: Option<String>,
    /// Called with the open menu's value, or `None`, whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<Option<String>>>,
    children: Children,
) -> impl IntoView {
    let input =
        Memo::new(move |_| MenubarInput { menus: menus.get(), open: open.get(), default_open: default_open.clone() });
    let state = RwSignal::new(input.with_untracked(MenubarMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| MenubarMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let ctx = BarCtx { id, state, input, on_open_change };
    provide_context(ctx);
    view! {
        <div
            role="menubar"
            aria-label=label
            aria-orientation="horizontal"
            data-scope=h::SCOPE
            data-part="root"
            on:keydown=move |ev| {
                if let Some(event) = h::root_keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {children()}
        </div>
    }
}

/// One menu in the bar: its trigger and its popover. Children are `MenubarItem`s.
#[component]
pub fn MenubarMenu(
    /// Unique key. Must match an entry in the bar's `menus`.
    #[prop(into)]
    value: String,
    /// Text of the trigger.
    #[prop(into)]
    label: String,
    /// The items, in display order.
    #[prop(into)]
    items: Signal<Vec<MenuEntry>>,
    /// Called with the activated item's value.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let bar = use_bar();
    let root = StoredValue::new(h::menu_root(&bar.id.get_value(), &value));
    let value = StoredValue::new(value);
    let is_open = Memo::new(move |_| bar.state.with(|s| s.open.value.as_deref() == Some(value.get_value().as_str())));
    let input = Memo::new(move |_| MenuInput { items: items.get(), open: Some(is_open.get()), default_open: false });
    let state = RwSignal::new(input.with_untracked(MenuMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| MenuMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });
    let ctx = MenuCtx { bar, root, value, state, input, on_select };
    // The popover and DOM focus follow the state. A menu opened by the bar has no
    // highlight yet, so `Home` picks the first item.
    Effect::new(move |_| {
        let (open, highlighted) = state.with(|s| (s.open.value, s.highlighted.clone()));
        let root = root.get_value();
        if open {
            show_popover(&m::content_id(&root));
            match highlighted {
                Some(item) => focus_id(&m::item_id(&root, &item)),
                None => ctx.send(MenuEvent::Home),
            }
        } else {
            hide_popover(&m::content_id(&root));
            if highlighted.is_some() {
                state.update(|s| s.highlighted = None);
            }
        }
    });
    provide_context(ctx);

    let trigger = Memo::new(move |_| state.with(|s| m::trigger_attrs(&root.get_value(), s)));
    let content = Memo::new(move |_| state.with(|s| m::content_attrs(&root.get_value(), s, m::DEFAULT_PLACEMENT)));
    let bar_attrs =
        Memo::new(move |_| bar.input.with(|i| bar.state.with(|s| h::trigger_attrs(s, i, &value.get_value()))));
    view! {
        <button
            type="button"
            id=move || trigger.with(|x| x.id.clone())
            role="menuitem"
            aria-haspopup="menu"
            aria-expanded=move || trigger.with(|x| x.aria_expanded).to_string()
            aria-controls=move || trigger.with(|x| x.aria_controls.clone())
            popovertarget=move || trigger.with(|x| x.aria_controls.clone())
            style=move || trigger.with(|x| x.style.clone())
            tabindex=move || bar_attrs.with(|x| x.tabindex)
            disabled=move || bar_attrs.with(|x| x.disabled)
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || bar_attrs.with(|x| x.data_state)
            on:click=move |ev| {
                // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                ev.prevent_default();
                bar.send(MenubarEvent::Toggle(value.get_value()));
            }
            on:keydown=move |ev| {
                if let Some(event) = h::trigger_keydown(&ev.key(), &value.get_value()) {
                    ev.prevent_default();
                    bar.send(event);
                }
            }
            on:pointerenter=move |_| bar.send(MenubarEvent::Hover(value.get_value()))
            on:focus=move |_| bar.send(MenubarEvent::Focus(value.get_value()))
        >
            {label}
        </button>
        <div
            id=move || content.with(|x| x.id.clone())
            role="menu"
            popover="auto"
            aria-labelledby=move || content.with(|x| x.aria_labelledby.clone())
            style=move || content.with(|x| x.style.clone())
            data-overlay=""
            data-side=move || content.with(|x| x.data_side)
            data-scope=m::SCOPE
            data-part="content"
            data-state=move || content.with(|x| x.data_state)
            on:keydown=move |ev| {
                if let Some((event, prevent)) = m::content_keydown(&ev.key()) {
                    if prevent {
                        ev.prevent_default();
                    }
                    ctx.send(event);
                }
            }
            on:toggle=move |ev| {
                // The browser closed it (outside press, or another menu opened): catch the machine up.
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

/// One action in a menu. Children are the label; `hint` shows right-aligned.
#[component]
pub fn MenubarItem(
    /// The item this renders. Must match an entry in the menu's `items`.
    #[prop(into)]
    value: String,
    /// Right-aligned hint, such as a keyboard shortcut.
    #[prop(optional, into)]
    hint: Option<String>,
    children: Children,
) -> impl IntoView {
    let ctx = use_menu();
    // Per item, only `highlighted` tracks the state; the id is fixed.
    let item_id = m::item_id(&ctx.root.get_value(), &value);
    let disabled = {
        let value = value.clone();
        Memo::new(move |_| ctx.input.with(|i| i.items.iter().any(|x| x.value == value && x.disabled)))
    };
    let highlighted = {
        let value = value.clone();
        Memo::new(move |_| ctx.state.with(|s| s.highlighted.as_deref() == Some(value.as_str())))
    };
    let (on_click, on_hover, on_focus) = (value.clone(), value.clone(), value);
    view! {
        <button
            type="button"
            id=item_id
            role="menuitem"
            tabindex="-1"
            disabled=move || disabled.get()
            data-scope=m::SCOPE
            data-part="item"
            data-highlighted=move || highlighted.get().then_some("")
            on:click=move |_| ctx.send(MenuEvent::Select(on_click.clone()))
            on:pointermove=move |_| {
                if !highlighted.get_untracked() {
                    ctx.send(MenuEvent::Highlight(on_hover.clone()));
                }
            }
            on:focus=move |_| ctx.send(MenuEvent::Highlight(on_focus.clone()))
        >
            <span data-scope=m::SCOPE data-part="item-label">{children()}</span>
            {hint.map(|hint| view! { <span data-scope=m::SCOPE data-part="item-hint">{hint}</span> })}
        </button>
    }
}
