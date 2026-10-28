use dioxus::prelude::*;
use hydroxy_dioxus::alert_dialog::AlertDialog;
use hydroxy_dioxus::button::ButtonVariant;

#[component]
pub fn AlertDialogDemo() -> Element {
    let mut deleted = use_signal(|| false);

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-5);",
            // A destructive confirmation. Cancel has focus when it opens.
            AlertDialog {
                trigger: "Delete project",
                title: "Delete Q3 customer research?",
                description: "This removes 12 files and 3 summaries. It can't be undone.",
                action_label: "Delete project",
                cancel_label: "Keep project",
                destructive: true,
                on_action: move |()| deleted.set(true),
            }
            // Not destructive: the confirm button is the primary look.
            AlertDialog {
                trigger: "Leave page",
                trigger_variant: ButtonVariant::Ghost,
                title: "Leave without sending?",
                description: "Your draft is saved, but it hasn't gone to anyone yet.",
                action_label: "Leave page",
            }
            span {
                if deleted() {
                    "Project deleted."
                } else {
                    "Nothing deleted yet."
                }
            }
        }
    }
}
