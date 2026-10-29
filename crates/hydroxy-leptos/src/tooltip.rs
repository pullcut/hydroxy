//! `<Tooltip>` / `<TooltipTrigger>` / `<TooltipContent>`.
//! The content is a manual popover in the top layer, anchored to the trigger
//! with CSS anchor positioning and shown or hidden to match the machine.
//!
//! Parts panic if rendered outside their root `Tooltip`.

use hydroxy_headless::tooltip::{self as h, TooltipCommand, TooltipEvent, TooltipInput, TooltipMachine, TooltipState};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::drive::drive;

use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::tooltip::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<TooltipState>,
    input: Memo<TooltipInput>,
    placement: Placement,
    on_open_change: Option<Callback<bool>>,
}

impl Ctx {
    fn send(self, event: TooltipEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        for command in drive::<TooltipMachine, _>(self.state, self.input, event) {
            match command {
                TooltipCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Tooltip parts must be rendered inside <Tooltip>")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Tooltip(
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Never open from hover or focus.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Where the tooltip sits. Defaults to above the trigger, centred.
    #[prop(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| TooltipInput { open: open.get(), default_open, disabled: disabled.get() });
    let state = RwSignal::new(input.with_untracked(TooltipMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| TooltipMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    // Mirror the open state onto the popover, whoever changed it.
    Effect::new(move |_| {
        let content = h::content_id(&id.get_value());
        if state.with(|s| s.open.value) { show_popover(&content) } else { hide_popover(&content) }
    });

    provide_context(Ctx { id, state, input, placement, on_open_change });
    children()
}

/// The button the tooltip describes. Hover or focus opens it; leave, blur or Esc closes it.
#[component]
pub fn TooltipTrigger(
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
            id=move || attrs.with(|x| x.id.clone())
            aria-describedby=move || attrs.with(|x| x.aria_describedby.clone())
            style=move || attrs.with(|x| x.style.clone())
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || attrs.with(|x| x.data_state)
            class=BUTTON_CLASS
            data-variant=variant.unwrap_or(ButtonVariant::Secondary).as_str()
            on:pointerenter=move |_| ctx.send(TooltipEvent::PointerEnter)
            on:pointerleave=move |_| ctx.send(TooltipEvent::PointerLeave)
            on:focus=move |_| ctx.send(TooltipEvent::Focus)
            on:blur=move |_| ctx.send(TooltipEvent::Blur)
            on:keydown=move |ev| {
                if let Some(event) = ctx.state.with_untracked(|s| h::keydown(&ev.key(), s)) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {children()}
        </button>
    }
}

/// The tooltip text. Always mounted; the popover API shows and hides it.
#[component]
pub fn TooltipContent(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, ctx.placement)));
    view! {
        <div
            id=move || attrs.with(|x| x.id.clone())
            role="tooltip"
            popover="manual"
            data-overlay=""
            style=move || attrs.with(|x| x.style.clone())
            data-side=move || attrs.with(|x| x.data_side)
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.with(|x| x.data_state)
        >
            {children()}
        </div>
    }
}
