use dioxus::prelude::*;
use oxide_dioxus::button::ButtonVariant;
use oxide_dioxus::dialog::{
    Dialog, DialogActions, DialogClose, DialogCloseIcon, DialogContent, DialogDescription, DialogTitle, DialogTrigger,
};

#[component]
pub fn DialogDemo() -> Element {
    let mut confirm_open = use_signal(|| Some(false));

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
            // Uncontrolled. Escape, the close icon or a press outside the panel closes it.
            Dialog {
                DialogTrigger { variant: ButtonVariant::Primary, "Share summary" }
                DialogContent {
                    DialogCloseIcon {}
                    DialogTitle { "Share the Q3 summary" }
                    DialogDescription {
                        "Everyone in the workspace can read it, with sources attached. You can stop sharing at any time."
                    }
                    DialogActions {
                        DialogClose { variant: ButtonVariant::Secondary, "Cancel" }
                        DialogClose { variant: ButtonVariant::Accent, "Share with workspace" }
                    }
                }
            }

            // Controlled alert: only its buttons close it.
            Dialog {
                alert: true,
                open: confirm_open(),
                on_open_change: move |open| confirm_open.set(Some(open)),
                DialogTrigger { variant: ButtonVariant::Secondary, "Delete project" }
                DialogContent {
                    DialogTitle { "Delete Q3 customer research?" }
                    DialogDescription { "This removes 12 files and 3 summaries. It can't be undone." }
                    DialogActions {
                        DialogClose { variant: ButtonVariant::Secondary, "Keep project" }
                        DialogClose { variant: ButtonVariant::Danger, "Delete project" }
                    }
                }
            }
        }
    }
}
