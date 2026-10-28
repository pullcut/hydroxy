//! `<Menu>` / `<MenuTrigger>` / `<MenuContent>` / `<MenuItem>`.
//! Parts read the machine from context and render the connector's attributes.
//! The content is a native `popover="auto"`: top layer, Esc and outside-click
//! dismissal come from the browser; CSS anchor positioning places it.

use leptos::prelude::*;
use oxide_headless::menu::{self as h, MenuCommand, MenuEvent, MenuInput, MenuMachine, MenuState};
use oxide_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};

use crate::focus::focus_id;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::menu::MenuEntry;
pub use oxide_headless::overlay::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<MenuState>,
    input: Memo<MenuInput>,
    placement: Placement,
    on_open_change: Option<Callback<bool>>,
    on_select: Option<Callback<String>>,
}

impl Ctx {
    fn send(self, event: MenuEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| MenuMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && self.state.with_untracked(|s| *s != next)
        {
            self.state.set(next);
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
                MenuCommand::FocusTrigger => focus_id(&h::trigger_id(&self.id.get_value())),
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Menu parts must be rendered inside <Menu>")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Menu(
    /// The items, in display order.
    #[prop(into)]
    items: Signal<Vec<MenuEntry>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Where the menu sits relative to the trigger. Default below, start-aligned.
    #[prop(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called when the menu opens or closes, or a change is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// Called with the activated item's value.
    #[prop(optional, into)]
    on_select: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| MenuInput { items: items.get(), open: open.get(), default_open });
    let state = RwSignal::new(input.with_untracked(MenuMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| MenuMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
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

    provide_context(Ctx { id, state, input, placement, on_open_change, on_select });
    children()
}

/// The button that opens the menu.
#[component]
pub fn MenuTrigger(
    /// Button look. Default secondary.
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s)));
    view! {
        <button
            type="button"
            id=move || attrs.get().id
            aria-haspopup="menu"
            aria-expanded=move || attrs.get().aria_expanded.to_string()
            aria-controls=move || attrs.get().aria_controls
            popovertarget=move || attrs.get().aria_controls
            style=move || attrs.get().style
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || attrs.get().data_state
            class=BUTTON_CLASS
            data-variant=variant.unwrap_or(ButtonVariant::Secondary).as_str()
            on:click=move |ev| {
                // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                ev.prevent_default();
                ctx.send(MenuEvent::Toggle);
            }
            on:keydown=move |ev| {
                if let Some(event) = h::trigger_keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {children()}
        </button>
    }
}

/// The floating list of items. An optional `label` shows as a small heading.
#[component]
pub fn MenuContent(
    /// Small uppercase heading above the items.
    #[prop(optional, into)]
    label: Option<String>,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, ctx.placement)));
    view! {
        <div
            id=move || attrs.get().id
            role="menu"
            popover="auto"
            aria-labelledby=move || attrs.get().aria_labelledby
            style=move || attrs.get().style
            data-overlay=""
            data-side=move || attrs.get().data_side
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.get().data_state
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
            {label.map(|label| view! { <div data-scope=h::SCOPE data-part="label" aria-hidden="true">{label}</div> })}
            {children()}
        </div>
    }
}

/// One action. Children are the label; `hint` shows right-aligned (a shortcut, say).
#[component]
pub fn MenuItem(
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
