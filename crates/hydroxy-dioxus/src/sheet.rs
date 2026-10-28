//! `Sheet` / `SheetTrigger` / `SheetContent` / `SheetHeader` / `SheetTitle` /
//! `SheetDescription` / `SheetBody` / `SheetFooter` / `SheetClose` / `SheetCloseIcon`.
//! A side drawer: a native `<dialog>` opened with `showModal()`, its panel
//! pinned to one edge of the viewport.

use dioxus::prelude::*;
use hydroxy_headless::sheet::{self as h, CloseReason, SheetCommand, SheetEvent, SheetInput, SheetMachine, SheetState};
use hydroxy_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::overlay::{close_dialog, show_modal};

pub use hydroxy_headless::sheet::SheetSide;

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<SheetState>,
    input: Memo<SheetInput>,
    on_open_change: Option<EventHandler<bool>>,
}

impl Ctx {
    fn send(mut self, event: SheetEvent) {
        let step = SheetMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                SheetCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Sheet parts must be rendered inside Sheet")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Sheet(
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let input = use_memo(move || SheetInput { open: open(), default_open, alert: false });
    let mut state = use_signal(|| SheetMachine::initial(&input.peek()));
    let id = use_hook(|| CopyValue::new(hydroxy_headless::next_id(h::SCOPE)));
    use_effect(move || {
        let synced = SheetMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    // Keep the native dialog in step with the machine.
    use_effect(move || {
        let content = h::content_id(&id.peek());
        if state.read().open {
            show_modal(&content);
        } else {
            close_dialog(&content);
        }
    });

    use_context_provider(|| Ctx { id, state, input, on_open_change });
    rsx! { {children} }
}

/// The button that opens the sheet. Set `variant` for the hydroxy button look.
#[component]
pub fn SheetTrigger(
    /// Gives the trigger the [`Button`](crate::button::Button) look in this variant.
    variant: Option<ButtonVariant>,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::trigger_attrs(&ctx.id.read(), &ctx.state.read());
    rsx! {
        button {
            r#type: "button",
            "aria-haspopup": a.aria_haspopup,
            "aria-expanded": a.aria_expanded.to_string(),
            "aria-controls": a.aria_controls,
            "data-scope": h::SCOPE,
            "data-part": "trigger",
            "data-state": a.data_state,
            class: variant.map(|_| BUTTON_CLASS),
            "data-variant": variant.map(ButtonVariant::as_str),
            onclick: move |_| ctx.send(SheetEvent::Open),
            {children}
        }
    }
}

/// The modal drawer: a full-viewport native `<dialog>` with the panel against
/// the `side` edge. A press outside the panel or Esc closes it.
#[component]
pub fn SheetContent(
    /// The edge the panel slides in from. Default the inline end (right in LTR).
    #[props(default)]
    side: SheetSide,
    children: Element,
) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read());
    rsx! {
        dialog {
            id: a.id,
            role: a.role,
            "aria-labelledby": a.aria_labelledby,
            "aria-describedby": a.aria_describedby,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
            "data-side": side.as_str(),
            "data-overlay": "true",
            onkeydown: move |e| {
                if let Some(event) = h::keydown(&e.key().to_string()) {
                    e.prevent_default();
                    ctx.send(event);
                }
            },
            // Other close requests (e.g. Android back): keep the machine in charge.
            oncancel: move |e| {
                e.prevent_default();
                ctx.send(SheetEvent::Close(CloseReason::Escape));
            },
            onpointerdown: move |e| {
                // Stops the press focusing the closing <dialog>, which would undo the
                // browser's focus return to the trigger.
                e.prevent_default();
                ctx.send(SheetEvent::Close(CloseReason::Outside));
            },
            div {
                "data-scope": h::SCOPE,
                "data-part": "panel",
                onpointerdown: move |e| e.stop_propagation(),
                {children}
            }
        }
    }
}

/// The top row: title, description and usually a [`SheetCloseIcon`].
#[component]
pub fn SheetHeader(children: Element) -> Element {
    rsx! {
        header { "data-scope": h::SCOPE, "data-part": "header", {children} }
    }
}

/// The sheet's heading. Labels the dialog.
#[component]
pub fn SheetTitle(children: Element) -> Element {
    let ctx = use_ctx();
    rsx! {
        h2 { id: h::title_id(&ctx.id.read()), "data-scope": h::SCOPE, "data-part": "title", {children} }
    }
}

/// Supporting text under the title. Describes the dialog.
#[component]
pub fn SheetDescription(children: Element) -> Element {
    let ctx = use_ctx();
    rsx! {
        p { id: h::description_id(&ctx.id.read()), "data-scope": h::SCOPE, "data-part": "description", {children} }
    }
}

/// The main content. Scrolls on its own when it overflows.
#[component]
pub fn SheetBody(children: Element) -> Element {
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "body", {children} }
    }
}

/// The bottom row of buttons, pinned below the body.
#[component]
pub fn SheetFooter(children: Element) -> Element {
    rsx! {
        footer { "data-scope": h::SCOPE, "data-part": "footer", {children} }
    }
}

/// A button that closes the sheet. Set `variant` for the hydroxy button look.
#[component]
pub fn SheetClose(
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
            onclick: move |_| ctx.send(SheetEvent::Close(CloseReason::CloseTrigger)),
            {children}
        }
    }
}

/// The icon-only close button. Place it in the [`SheetHeader`].
#[component]
pub fn SheetCloseIcon() -> Element {
    let ctx = use_ctx();
    rsx! {
        button {
            r#type: "button",
            "aria-label": "Close",
            title: "Close",
            class: ICON_BUTTON_CLASS,
            "data-variant": "ghost",
            "data-size": "sm",
            "data-scope": h::SCOPE,
            "data-part": "close-icon",
            onclick: move |_| ctx.send(SheetEvent::Close(CloseReason::CloseTrigger)),
            Icon { name: IconName::X, size: 14 }
        }
    }
}
