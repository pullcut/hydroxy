//! `AlertDialog`: a confirmation that interrupts, built on `Dialog`.
//! It has `role="alertdialog"` and ignores Escape and backdrop presses, so it
//! closes only through its two buttons. Cancel comes first in the DOM, so the
//! browser focuses it when the dialog opens: the safe choice is the default.

use dioxus::prelude::*;

use crate::button::{BUTTON_CLASS, ButtonVariant};
use crate::dialog::{Dialog, DialogActions, DialogClose, DialogContent, DialogDescription, DialogTitle, DialogTrigger};

/// A modal confirmation with a title, a description and two buttons.
#[component]
pub fn AlertDialog(
    /// The question, as a heading.
    title: String,
    /// What confirming does.
    description: String,
    /// Label of the button that opens the dialog. Leave out to open it from code with `open`.
    trigger: Option<String>,
    /// Look of the trigger button.
    #[props(default = ButtonVariant::Secondary)]
    trigger_variant: ButtonVariant,
    /// Label of the cancel button.
    #[props(default = "Cancel".to_string())]
    cancel_label: String,
    /// Label of the confirm button.
    #[props(default = "Continue".to_string())]
    action_label: String,
    /// Style the confirm button as destructive.
    #[props(default)]
    destructive: bool,
    /// Called when the confirm button is pressed.
    on_action: Option<EventHandler<()>>,
    /// Controlled open state. Pair with `on_open_change`.
    #[props(default)]
    open: ReadSignal<Option<bool>>,
    /// Initial open state when uncontrolled.
    #[props(default)]
    default_open: bool,
    /// Called with the new open state whenever it changes or is requested.
    on_open_change: Option<EventHandler<bool>>,
    /// Extra content between the description and the buttons.
    children: Element,
) -> Element {
    // The inner dialog is always controlled; this keeps the uncontrolled flag here.
    let mut inner = use_signal(|| default_open);
    let is_open = use_memo(move || Some(open().unwrap_or_else(&*inner)));
    let change = use_callback(move |next: bool| {
        inner.set(next);
        if let Some(handler) = on_open_change {
            handler.call(next);
        }
    });
    let action_variant = if destructive { ButtonVariant::Danger } else { ButtonVariant::Primary };

    rsx! {
        Dialog { alert: true, open: is_open, on_open_change: change,
            if let Some(label) = trigger {
                DialogTrigger { variant: trigger_variant, "{label}" }
            }
            DialogContent {
                DialogTitle { "{title}" }
                DialogDescription { "{description}" }
                {children}
                DialogActions {
                    DialogClose { variant: ButtonVariant::Secondary, "{cancel_label}" }
                    button {
                        r#type: "button",
                        class: BUTTON_CLASS,
                        "data-variant": action_variant.as_str(),
                        "data-scope": "alert-dialog",
                        "data-part": "action",
                        onclick: move |_| {
                            if let Some(handler) = on_action {
                                handler.call(());
                            }
                            change.call(false);
                        },
                        "{action_label}"
                    }
                }
            }
        }
    }
}
