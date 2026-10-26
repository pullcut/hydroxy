//! `<Dialog>` / `<DialogTrigger>` / `<DialogContent>` / `<DialogTitle>` /
//! `<DialogDescription>` / `<DialogActions>` / `<DialogClose>` / `<DialogCloseIcon>`.
//! The content is a native `<dialog>` opened with `showModal()`.

use leptos::prelude::*;
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
    id: StoredValue<String>,
    state: RwSignal<DialogState>,
    input: Memo<DialogInput>,
    on_open_change: Option<Callback<bool>>,
}

impl Ctx {
    fn send(self, event: DialogEvent) {
        // Events can still arrive after unmount (blur or toggle on a removed element).
        let Some(step) = self
            .input
            .try_with_untracked(|input| {
                self.state.try_with_untracked(|state| DialogMachine::transition(state, event, input))
            })
            .flatten()
        else {
            return;
        };
        self.state.set(step.state);
        for command in step.commands {
            match command {
                DialogCommand::OpenChange(open) => {
                    if let Some(cb) = self.on_open_change {
                        cb.run(open);
                    }
                }
            }
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>().expect("Dialog parts must be rendered inside <Dialog>")
}

/// Root. Owns the machine and provides it to the parts below. Renders no element.
#[component]
pub fn Dialog(
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// An alert dialog (`role="alertdialog"`): Escape and backdrop clicks don't close it.
    #[prop(optional)]
    alert: bool,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| DialogInput { open: open.get(), default_open, alert });
    let state = RwSignal::new(input.with_untracked(DialogMachine::initial));
    let id = StoredValue::new(oxide_headless::next_id(h::SCOPE));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DialogMachine::sync(s, i)));
        state.set(synced);
    });
    // Keep the native dialog in step with the machine.
    Effect::new(move |_| {
        let content = h::content_id(&id.get_value());
        if state.with(|s| s.open) {
            show_modal(&content);
        } else {
            close_dialog(&content);
        }
    });

    provide_context(Ctx { id, state, input, on_open_change });
    children()
}

/// The button that opens the dialog. Set `variant` for the oxide button look.
#[component]
pub fn DialogTrigger(
    /// Gives the trigger the [`Button`](crate::button::Button) look in this variant.
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    children: Children,
) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.state.with(|s| h::trigger_attrs(&ctx.id.get_value(), s)));
    view! {
        <button
            type="button"
            aria-haspopup="dialog"
            aria-expanded=move || attrs.get().aria_expanded.to_string()
            aria-controls=move || attrs.get().aria_controls
            data-scope=h::SCOPE
            data-part="trigger"
            data-state=move || attrs.get().data_state
            class=variant.map(|_| BUTTON_CLASS)
            data-variant=variant.map(ButtonVariant::as_str)
            on:click=move |_| ctx.send(DialogEvent::Open)
        >
            {children()}
        </button>
    }
}

/// The modal: a full-viewport native `<dialog>` with the panel centred inside.
/// A press outside the panel or Escape closes it, unless it's an alert.
#[component]
pub fn DialogContent(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    let attrs = Memo::new(move |_| ctx.input.with(|i| ctx.state.with(|s| h::content_attrs(&ctx.id.get_value(), s, i))));
    view! {
        <dialog
            id=move || attrs.get().id
            role=move || attrs.get().role
            aria-labelledby=move || attrs.get().aria_labelledby
            aria-describedby=move || attrs.get().aria_describedby
            data-scope=h::SCOPE
            data-part="content"
            data-state=move || attrs.get().data_state
            data-overlay="true"
            on:keydown=move |ev| {
                if let Some(event) = h::keydown(&ev.key()) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
            // Other close requests (e.g. Android back): keep the machine in charge.
            on:cancel=move |ev: leptos::ev::Event| {
                ev.prevent_default();
                ctx.send(DialogEvent::Close(CloseReason::Escape));
            }
            on:pointerdown=move |ev| {
                // Stops the press focusing the closing <dialog>, which would undo the
                // browser's focus return to the trigger.
                ev.prevent_default();
                ctx.send(DialogEvent::Close(CloseReason::Outside));
            }
        >
            <div data-scope=h::SCOPE data-part="panel" on:pointerdown=|ev| ev.stop_propagation()>
                {children()}
            </div>
        </dialog>
    }
}

/// The dialog's heading. Labels the dialog.
#[component]
pub fn DialogTitle(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <h2 id=h::title_id(&ctx.id.get_value()) data-scope=h::SCOPE data-part="title">
            {children()}
        </h2>
    }
}

/// Supporting text. Describes the dialog.
#[component]
pub fn DialogDescription(children: Children) -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <p id=h::description_id(&ctx.id.get_value()) data-scope=h::SCOPE data-part="description">
            {children()}
        </p>
    }
}

/// The right-aligned row of buttons at the bottom.
#[component]
pub fn DialogActions(children: Children) -> impl IntoView {
    view! { <div data-scope=h::SCOPE data-part="actions">{children()}</div> }
}

/// A button that closes the dialog. Works in alert dialogs too.
/// Set `variant` for the oxide button look.
#[component]
pub fn DialogClose(
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
            on:click=move |_| ctx.send(DialogEvent::Close(CloseReason::CloseTrigger))
        >
            {children()}
        </button>
    }
}

/// The icon-only close button in the panel's top corner.
#[component]
pub fn DialogCloseIcon() -> impl IntoView {
    let ctx = use_ctx();
    view! {
        <button
            type="button"
            aria-label="Close"
            title="Close"
            class=ICON_BUTTON_CLASS
            data-variant="ghost"
            data-size="sm"
            data-scope=h::SCOPE
            data-part="close-icon"
            on:click=move |_| ctx.send(DialogEvent::Close(CloseReason::CloseTrigger))
        >
            <Icon name=IconName::X size=14 />
        </button>
    }
}
