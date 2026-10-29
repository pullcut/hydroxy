//! `HoverCard` / `HoverCardTrigger` / `HoverCardContent`.
//! A preview card for a link. The content is a manual popover in the top layer,
//! anchored to the trigger with CSS anchor positioning.
//!
//! Parts panic if rendered outside their root `HoverCard`.

use dioxus::prelude::*;
use hydroxy_headless::hover_card::{
    self as h, HoverCardCommand, HoverCardEvent, HoverCardInput, HoverCardMachine, HoverCardState,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::js;
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::hover_card::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<HoverCardState>,
    input: Memo<HoverCardInput>,
    placement: Placement,
    on_open_change: Option<EventHandler<bool>>,
}

impl Ctx {
    fn send(self, event: HoverCardEvent) {
        for command in drive::<HoverCardMachine>(self.state, self.input, event) {
            match command {
                HoverCardCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
                // The clock runs through eval, so it works on web and desktop alike.
                // Stale waits are ignored by the machine, so there is nothing to cancel.
                HoverCardCommand::Wait { id, ms } => {
                    spawn(async move {
                        js::sleep(ms).await;
                        self.send(HoverCardEvent::Elapsed(id));
                    });
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("HoverCard parts must be rendered inside HoverCard")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn HoverCard(
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Milliseconds the pointer rests on the trigger before the card opens.
    #[props(default = 300)]
    open_delay_ms: u32,
    /// Milliseconds after the pointer leaves trigger and card before it closes.
    #[props(default = 150)]
    close_delay_ms: u32,
    /// Where the card sits. Defaults to below the trigger, centred.
    #[props(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let input = use_memo(move || HoverCardInput { open: open(), default_open, open_delay_ms, close_delay_ms });
    let mut state = use_signal(|| HoverCardMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = HoverCardMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let id = use_hook(|| CopyValue::new(hydroxy_headless::next_id(h::SCOPE)));
    // Mirror the open state onto the popover, whoever changed it.
    let open_now = use_memo(move || state.read().open.value);
    use_effect(move || {
        let content = h::content_id(&id.peek());
        if open_now() { show_popover(&content) } else { hide_popover(&content) }
    });

    use_context_provider(|| Ctx { id, state, input, placement, on_open_change });
    rsx! { {children} }
}

/// The link the card previews. Hover (after the delay) or focus opens the card.
#[component]
pub fn HoverCardTrigger(
    /// Where the link goes.
    href: String,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::trigger_attrs(&ctx.id.read(), &ctx.state.read());
    rsx! {
        a {
            href,
            "aria-expanded": a.aria_expanded.to_string(),
            "aria-controls": a.aria_controls,
            style: a.style,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": a.data_state,
            onpointerenter: move |_| ctx.send(HoverCardEvent::PointerEnter),
            onpointerleave: move |_| ctx.send(HoverCardEvent::PointerLeave),
            onfocus: move |_| ctx.send(HoverCardEvent::Focus),
            onblur: move |_| ctx.send(HoverCardEvent::Blur),
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

/// The card. Always mounted; the popover API shows and hides it. Keeps itself
/// open while the pointer is over it. Renders a `<div>`, so keep it out of `<p>`.
#[component]
pub fn HoverCardContent(children: Element) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read(), ctx.placement);
    rsx! {
        div {
            id: a.id,
            "popover": "manual",
            "data-overlay": "",
            style: a.style,
            "data-side": a.data_side,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            onpointerenter: move |_| ctx.send(HoverCardEvent::PointerEnter),
            onpointerleave: move |_| ctx.send(HoverCardEvent::PointerLeave),
            {children}
        }
    }
}
