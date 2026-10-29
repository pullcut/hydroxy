//! `<HoverCard>` / `<HoverCardTrigger>` / `<HoverCardContent>`.
//! A preview card for a link. The content is a manual popover in the top layer,
//! anchored to the trigger with CSS anchor positioning.
//!
//! Parts panic if rendered outside their root `HoverCard`.

use crate::drive::drive;
use std::time::Duration;

use hydroxy_headless::hover_card::{
    self as h, HoverCardCommand, HoverCardEvent, HoverCardInput, HoverCardMachine, HoverCardState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::hover_card::{Align, Placement, Side};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<HoverCardState>,
    input: Memo<HoverCardInput>,
    placement: Placement,
    on_open_change: Option<Callback<bool>>,
}

impl Ctx {
    fn send(self, event: HoverCardEvent) {
        // Events can still arrive after unmount (a timer, or blur on a removed element).
        for command in drive::<HoverCardMachine, _>(self.state, self.input, event) {
            match command {
                HoverCardCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
                // Stale waits are ignored by the machine, so there is nothing to cancel.
                HoverCardCommand::Wait { id, ms } => {
                    set_timeout(move || self.send(HoverCardEvent::Elapsed(id)), Duration::from_millis(ms.into()));
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("HoverCard parts must be rendered inside <HoverCard>")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn HoverCard(
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Milliseconds the pointer rests on the trigger before the card opens.
    #[prop(default = 300)]
    open_delay_ms: u32,
    /// Milliseconds after the pointer leaves trigger and card before it closes.
    #[prop(default = 150)]
    close_delay_ms: u32,
    /// Where the card sits. Defaults to below the trigger, centred.
    #[prop(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| HoverCardInput { open: open.get(), default_open, open_delay_ms, close_delay_ms });
    let state = RwSignal::new(input.with_untracked(HoverCardMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| HoverCardMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    // Mirror the open state onto the popover, whoever changed it.
    let open_now = Memo::new(move |_| state.with(|s| s.open.value));
    Effect::new(move |_| {
        let content = h::content_id(&id.get_value());
        if open_now.get() { show_popover(&content) } else { hide_popover(&content) }
    });

    provide_context(Ctx { id, state, input, placement, on_open_change });
    children()
}

/// The link the card previews. Hover (after the delay) or focus opens the card.
#[component]
pub fn HoverCardTrigger(
    /// Where the link goes.
    #[prop(into)]
    href: String,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s)));
    view! {
        <a
            href=href
            aria-expanded=move || attrs.with(|x| x.aria_expanded).to_string()
            aria-controls=move || attrs.with(|x| x.aria_controls.clone())
            style=move || attrs.with(|x| x.style.clone())
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || attrs.with(|x| x.data_state)
            on:pointerenter=move |_| ctx.send(HoverCardEvent::PointerEnter)
            on:pointerleave=move |_| ctx.send(HoverCardEvent::PointerLeave)
            on:focus=move |_| ctx.send(HoverCardEvent::Focus)
            on:blur=move |_| ctx.send(HoverCardEvent::Blur)
            on:keydown=move |ev| {
                if let Some(event) = ctx.state.with_untracked(|s| h::keydown(&ev.key(), s)) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {children()}
        </a>
    }
}

/// The card. Always mounted; the popover API shows and hides it. Keeps itself
/// open while the pointer is over it. Renders a `<div>`, so keep it out of `<p>`.
#[component]
pub fn HoverCardContent(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, ctx.placement)));
    view! {
        <div
            id=move || attrs.with(|x| x.id.clone())
            popover="manual"
            data-overlay=""
            style=move || attrs.with(|x| x.style.clone())
            data-side=move || attrs.with(|x| x.data_side)
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.with(|x| x.data_state)
            on:pointerenter=move |_| ctx.send(HoverCardEvent::PointerEnter)
            on:pointerleave=move |_| ctx.send(HoverCardEvent::PointerLeave)
        >
            {children()}
        </div>
    }
}
