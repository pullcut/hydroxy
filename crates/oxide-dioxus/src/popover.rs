//! `Popover` / `PopoverTrigger` / `PopoverContent` / `PopoverClose`.
//! A non-modal panel anchored to its trigger. The content is a native
//! `popover="auto"`: top layer and outside-click dismissal come from the browser;
//! CSS anchor positioning places it.

use dioxus::prelude::*;
use oxide_headless::popover::{self as h, PopoverCommand, PopoverEvent, PopoverInput, PopoverMachine, PopoverState};
use oxide_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::focus::FocusMap;
use crate::overlay::{hide_popover, show_popover};

pub use oxide_headless::overlay::{Align, Placement, Side};

/// `FocusMap` key for the trigger.
const TRIGGER_KEY: &str = "trigger";

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<PopoverState>,
    input: Memo<PopoverInput>,
    placement: Placement,
    focus: FocusMap,
    on_open_change: Option<EventHandler<bool>>,
}

impl Ctx {
    fn send(mut self, event: PopoverEvent) {
        let step = PopoverMachine::transition(&self.state.peek(), event, &self.input.peek());
        if *self.state.peek() != step.state {
            self.state.set(step.state);
        }
        for command in step.commands {
            match command {
                PopoverCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
                PopoverCommand::FocusTrigger => self.focus.focus(TRIGGER_KEY),
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Popover parts must be rendered inside Popover")
}

/// Focus the first focusable element in the content, or the content itself.
fn focus_first(content_id: &str) {
    let _ = document::eval(&format!(
        "const c = document.getElementById({content_id:?}); if (c) (c.querySelector({sel:?}) || c).focus();",
        sel = h::FOCUSABLE
    ));
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Popover(
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Where the panel sits relative to the trigger. Default below, centred.
    #[props(default = h::DEFAULT_PLACEMENT)]
    placement: Placement,
    /// Called when the panel opens or closes, or a change is requested.
    on_open_change: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let input = use_memo(move || PopoverInput { open: open(), default_open });
    let mut state = use_signal(|| PopoverMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = PopoverMachine::sync(&state.peek(), &input.read());
        if *state.peek() != synced {
            state.set(synced);
        }
    });

    let ctx = use_context_provider(|| Ctx {
        id: CopyValue::new(oxide_headless::next_id(h::SCOPE)),
        state,
        input,
        placement,
        focus: FocusMap::new(),
        on_open_change,
    });

    // The native popover follows the state, so controlled changes land too.
    use_effect(move || {
        let content = h::content_id(&ctx.id.peek());
        if state.read().open {
            show_popover(&content);
            focus_first(&content);
        } else {
            hide_popover(&content);
        }
    });

    rsx! { {children} }
}

/// The button that opens the panel.
#[component]
pub fn PopoverTrigger(
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
            "aria-haspopup": "dialog",
            "aria-expanded": a.aria_expanded.to_string(),
            "aria-controls": a.aria_controls.clone(),
            "popovertarget": a.aria_controls,
            style: a.style,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": a.data_state,
            class: BUTTON_CLASS,
            "data-variant": variant.unwrap_or(ButtonVariant::Secondary).as_str(),
            onmounted: move |e| ctx.focus.register(TRIGGER_KEY.to_string(), e.data()),
            onclick: move |e| {
                // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                e.prevent_default();
                ctx.send(PopoverEvent::Toggle);
            },
            {children}
        }
    }
}

/// The floating panel. Gets focus (or its first focusable child) on open.
#[component]
pub fn PopoverContent(children: Element) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read(), ctx.placement);
    let content = a.id.clone();
    rsx! {
        div {
            id: a.id,
            role: "dialog",
            "popover": "auto",
            tabindex: "-1",
            "aria-labelledby": a.aria_labelledby,
            style: a.style,
            "data-overlay": "",
            "data-side": a.data_side,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            onkeydown: move |e| {
                if let Some(event) = h::content_keydown(&e.key().to_string()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            ontoggle: move |_| {
                // The browser may have closed it (outside click): ask, then catch the machine up.
                let js = format!("return document.getElementById({content:?})?.matches(':popover-open') ?? false");
                spawn(async move {
                    if let Ok(false) = document::eval(&js).join::<bool>().await {
                        ctx.send(PopoverEvent::Close { restore_focus: false });
                    }
                });
            },
            {children}
        }
    }
}

/// A button inside the panel that closes it and returns focus to the trigger.
#[component]
pub fn PopoverClose(
    /// Gives the button the [`Button`](crate::button::Button) look in this variant.
    variant: Option<ButtonVariant>,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    rsx! {
        button {
            r#type: "button",
            "data-scope": h::SCOPE,
            "data-part": "close",
            class: variant.map(|_| BUTTON_CLASS),
            "data-variant": variant.map(ButtonVariant::as_str),
            onclick: move |_| ctx.send(PopoverEvent::Close { restore_focus: true }),
            {children}
        }
    }
}
