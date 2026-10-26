//! `Dialog` / `DialogTrigger` / `DialogContent` / `DialogTitle` /
//! `DialogDescription` / `DialogActions` / `DialogClose` / `DialogCloseIcon`.
//! The content is a native `<dialog>` opened with `showModal()`.

use dioxus::prelude::*;
use oxide_headless::dialog::{
    self as h, CloseReason, DialogCommand, DialogEvent, DialogInput, DialogMachine, DialogState,
};
use oxide_statechart::Machine;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::icon::{Icon, IconName};
use crate::icon_button::ICON_BUTTON_CLASS;
use crate::overlay::{close_dialog, show_modal};

#[derive(Clone, Copy)]
struct Ctx {
    id: CopyValue<String>,
    state: Signal<DialogState>,
    input: Memo<DialogInput>,
    on_open_change: Option<EventHandler<bool>>,
}

impl Ctx {
    fn send(mut self, event: DialogEvent) {
        let step = DialogMachine::transition(&self.state.peek(), event, &self.input.peek());
        self.state.set(step.state);
        for command in step.commands {
            match command {
                DialogCommand::OpenChange(open) => {
                    if let Some(handler) = self.on_open_change {
                        handler.call(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    try_use_context::<Ctx>().expect("Dialog parts must be rendered inside Dialog")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Dialog(
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// An alert dialog (`role="alertdialog"`): Escape and backdrop clicks don't close it.
    #[props(default)]
    alert: bool,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let input = use_memo(move || DialogInput { open: open(), default_open, alert });
    let mut state = use_signal(|| DialogMachine::initial(&input.peek()));
    let id = use_hook(|| CopyValue::new(oxide_headless::next_id(h::SCOPE)));
    use_effect(move || {
        let synced = DialogMachine::sync(&state.peek(), &input.read());
        state.set(synced);
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

/// The button that opens the dialog. Set `variant` for the oxide button look.
#[component]
pub fn DialogTrigger(
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
            onclick: move |_| ctx.send(DialogEvent::Open),
            {children}
        }
    }
}

/// The modal: a full-viewport native `<dialog>` with the panel centred inside.
/// A press outside the panel or Escape closes it, unless it's an alert.
#[component]
pub fn DialogContent(children: Element) -> Element {
    let ctx = use_ctx();
    let a = h::content_attrs(&ctx.id.read(), &ctx.state.read(), &ctx.input.read());
    rsx! {
        dialog {
            id: a.id,
            role: a.role,
            "aria-labelledby": a.aria_labelledby,
            "aria-describedby": a.aria_describedby,
            "data-scope": h::SCOPE,
            "data-part": "content",
            "data-state": a.data_state,
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
                ctx.send(DialogEvent::Close(CloseReason::Escape));
            },
            onpointerdown: move |e| {
                // Stops the press focusing the closing <dialog>, which would undo the
                // browser's focus return to the trigger.
                e.prevent_default();
                ctx.send(DialogEvent::Close(CloseReason::Outside));
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

/// The dialog's heading. Labels the dialog.
#[component]
pub fn DialogTitle(children: Element) -> Element {
    let ctx = use_ctx();
    rsx! {
        h2 { id: h::title_id(&ctx.id.read()), "data-scope": h::SCOPE, "data-part": "title", {children} }
    }
}

/// Supporting text. Describes the dialog.
#[component]
pub fn DialogDescription(children: Element) -> Element {
    let ctx = use_ctx();
    rsx! {
        p { id: h::description_id(&ctx.id.read()), "data-scope": h::SCOPE, "data-part": "description", {children} }
    }
}

/// The right-aligned row of buttons at the bottom.
#[component]
pub fn DialogActions(children: Element) -> Element {
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "actions", {children} }
    }
}

/// A button that closes the dialog. Works in alert dialogs too.
/// Set `variant` for the oxide button look.
#[component]
pub fn DialogClose(
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
            onclick: move |_| ctx.send(DialogEvent::Close(CloseReason::CloseTrigger)),
            {children}
        }
    }
}

/// The icon-only close button in the panel's top corner.
#[component]
pub fn DialogCloseIcon() -> Element {
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
            onclick: move |_| ctx.send(DialogEvent::Close(CloseReason::CloseTrigger)),
            Icon { name: IconName::X, size: 14 }
        }
    }
}
