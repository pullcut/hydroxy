//! `<Popover>` / `<PopoverTrigger>` / `<PopoverContent>` / `<PopoverClose>`.
//! A non-modal panel anchored to its trigger. The content is a native
//! `popover="auto"`: top layer and outside-click dismissal come from the browser;
//! CSS anchor positioning places it.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::popover::{self as h, PopoverCommand, PopoverEvent, PopoverInput, PopoverMachine, PopoverState};
use oxide_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::focus::focus_id;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::overlay::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<PopoverState>,
    input: Memo<PopoverInput>,
    placement: Placement,
    on_open_change: Option<Callback<bool>>,
}

impl Ctx {
    fn send(self, event: PopoverEvent) {
        // Events can still arrive after unmount (toggle on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| PopoverMachine::transition(state, event, input))
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
                PopoverCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
                PopoverCommand::FocusTrigger => focus_id(&h::trigger_id(&self.id.get_value())),
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Popover parts must be rendered inside <Popover>")
}

/// Focus the first focusable element in the content, or the content itself.
fn focus_first(content_id: &str) {
    let Some(content) = document().get_element_by_id(content_id) else { return };
    let target = content.query_selector(h::FOCUSABLE).ok().flatten().unwrap_or(content);
    let _ = target.unchecked_into::<web_sys::HtmlElement>().focus();
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Popover(
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Where the panel sits relative to the trigger. Default below, centred.
    #[prop(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called when the panel opens or closes, or a change is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| PopoverInput { open: open.get(), default_open });
    let state = RwSignal::new(input.with_untracked(PopoverMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| PopoverMachine::sync(s, i)));
        if state.with_untracked(|s| *s != synced) {
            state.set(synced);
        }
    });

    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    // The native popover follows the state, so controlled changes land too.
    Effect::new(move |_| {
        let content = h::content_id(&id.get_value());
        if state.with(|s| s.open) {
            show_popover(&content);
            focus_first(&content);
        } else {
            hide_popover(&content);
        }
    });

    provide_context(Ctx { id, state, input, placement, on_open_change });
    children()
}

/// The button that opens the panel.
#[component]
pub fn PopoverTrigger(
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
            aria-haspopup="dialog"
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
                ctx.send(PopoverEvent::Toggle);
            }
        >
            {children()}
        </button>
    }
}

/// The floating panel. Gets focus (or its first focusable child) on open.
#[component]
pub fn PopoverContent(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, ctx.placement)));
    view! {
        <div
            id=move || attrs.get().id
            role="dialog"
            popover="auto"
            tabindex="-1"
            aria-labelledby=move || attrs.get().aria_labelledby
            style=move || attrs.get().style
            data-overlay=""
            data-side=move || attrs.get().data_side
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.get().data_state
            on:keydown=move |ev| {
                if let Some(event) = h::content_keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
            on:toggle=move |ev| {
                // The browser closed it (outside click): catch the machine up.
                let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                if !showing {
                    ctx.send(PopoverEvent::Close { restore_focus: false });
                }
            }
        >
            {children()}
        </div>
    }
}

/// A button inside the panel that closes it and returns focus to the trigger.
#[component]
pub fn PopoverClose(
    /// Gives the button the [`Button`](crate::button::Button) look in this variant.
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <button
            type="button"
            data-scope=h::SCOPE
            data-part="close"
            class=variant.map(|_| BUTTON_CLASS)
            data-variant=variant.map(ButtonVariant::as_str)
            on:click=move |_| ctx.send(PopoverEvent::Close { restore_focus: true })
        >
            {children()}
        </button>
    }
}
