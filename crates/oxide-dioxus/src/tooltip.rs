//! `Tooltip` / `TooltipTrigger` / `TooltipContent`.
//! The content is a manual popover in the top layer, anchored to the trigger
//! with CSS anchor positioning and shown or hidden to match the machine.

use dioxus::prelude::*;
use oxide_headless::tooltip::{self as h, TooltipCommand, TooltipEvent, TooltipInput, TooltipMachine, TooltipState};
use oxide_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};

use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::tooltip::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<TooltipState>,
    input: Memo<TooltipInput>,
    placement: Placement,
    on_open_change: Option<EventHandler<bool>>,
}

impl Ctx {
    fn send(mut self, event: TooltipEvent) {
        let step = TooltipMachine::transition(&self.state.peek(), event, &self.input.peek());
        self.state.set(step.state);
        for command in step.commands {
            match command {
                TooltipCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Tooltip parts must be rendered inside Tooltip")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Tooltip(
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Never open from hover or focus.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Where the tooltip sits. Defaults to above the trigger, centred.
    #[props(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let input = use_memo(move || TooltipInput { open: open(), default_open, disabled: disabled() });
    let mut state = use_signal(|| TooltipMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = TooltipMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });

    let id = use_hook(|| CopyValue::new(oxide_headless::next_id(h::SCOPE)));
    // Mirror the open state onto the popover, whoever changed it.
    use_effect(move || {
        let content = h::content_id(&id.read());
        if state.read().open { show_popover(&content) } else { hide_popover(&content) }
    });

    use_context_provider(|| Ctx { id, state, input, placement, on_open_change });
    rsx! { {children} }
}

/// The button the tooltip describes. Hover or focus opens it; leave, blur or Esc closes it.
#[component]
pub fn TooltipTrigger(
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
            "aria-describedby": a.aria_describedby,
            style: a.style,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": a.data_state,
            class: BUTTON_CLASS,
            "data-variant": variant.unwrap_or(ButtonVariant::Secondary).as_str(),
            onpointerenter: move |_| ctx.send(TooltipEvent::PointerEnter),
            onpointerleave: move |_| ctx.send(TooltipEvent::PointerLeave),
            onfocus: move |_| ctx.send(TooltipEvent::Focus),
            onblur: move |_| ctx.send(TooltipEvent::Blur),
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string(), &ctx.state.peek()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            {children}
        }
    }
}

/// The tooltip text. Always mounted; the popover API shows and hides it.
#[component]
pub fn TooltipContent(children: Element) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read(), ctx.placement);
    rsx! {
        div {
            id: a.id,
            role: a.role,
            "popover": a.popover,
            "data-overlay": "",
            style: a.style,
            "data-side": a.data_side,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            {children}
        }
    }
}
