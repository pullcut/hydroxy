//! `<AlertDialog>`: a confirmation that interrupts, built on [`Dialog`].
//! It has `role="alertdialog"` and ignores Escape and backdrop presses, so it
//! closes only through its two buttons. Cancel comes first in the DOM, so the
//! browser focuses it when the dialog opens: the safe choice is the default.

use leptos::prelude::*;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::dialog::{Dialog, DialogActions, DialogClose, DialogContent, DialogDescription, DialogTitle, DialogTrigger};

/// A modal confirmation with a title, a description and two buttons.
#[component]
pub fn AlertDialog(
    /// The question, as a heading.
    #[prop(into)]
    title: String,
    /// What confirming does.
    #[prop(into)]
    description: String,
    /// Label of the button that opens the dialog. Leave out to open it from code with `open`.
    #[prop(optional, into)]
    trigger: Option<String>,
    /// Look of the trigger button.
    #[prop(default = ButtonVariant::Secondary)]
    trigger_variant: ButtonVariant,
    /// Label of the cancel button.
    #[prop(default = "Cancel".into(), into)]
    cancel_label: String,
    /// Label of the confirm button.
    #[prop(default = "Continue".into(), into)]
    action_label: String,
    /// Style the confirm button as destructive.
    #[prop(optional)]
    destructive: bool,
    /// Called when the confirm button is pressed.
    #[prop(optional, into)]
    on_action: Option<Callback<()>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[prop(optional, into)]
    open: MaybeProp<bool>,
    /// Initial open state when uncontrolled.
    #[prop(optional)]
    default_open: bool,
    /// Called with the new open state whenever it changes or is requested.
    #[prop(optional, into)]
    on_open_change: Option<Callback<bool>>,
    /// Extra content between the description and the buttons.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    // The inner dialog is always controlled; this keeps the uncontrolled flag here.
    let inner = RwSignal::new(default_open);
    let is_open = Signal::derive(move || open.get().unwrap_or_else(|| inner.get()));
    let change = Callback::new(move |next: bool| {
        inner.set(next);
        if let Some(cb) = on_open_change {
            cb.run(next);
        }
    });
    let confirm = move |_| {
        if let Some(cb) = on_action {
            cb.run(());
        }
        change.run(false);
    };
    let action_variant = if destructive { ButtonVariant::Danger } else { ButtonVariant::Primary };

    view! {
        <Dialog alert=true open=is_open on_open_change=change>
            {trigger.map(|label| view! { <DialogTrigger variant=trigger_variant>{label}</DialogTrigger> })}
            <DialogContent>
                <DialogTitle>{title}</DialogTitle>
                <DialogDescription>{description}</DialogDescription>
                {children.map(|c| c())}
                <DialogActions>
                    <DialogClose variant=ButtonVariant::Secondary>{cancel_label}</DialogClose>
                    <button
                        type="button"
                        class=BUTTON_CLASS
                        data-variant=action_variant.as_str()
                        data-scope="alert-dialog"
                        data-part="action"
                        on:click=confirm
                    >
                        {action_label}
                    </button>
                </DialogActions>
            </DialogContent>
        </Dialog>
    }
}
