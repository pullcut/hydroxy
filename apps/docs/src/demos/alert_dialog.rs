use hydroxy_leptos::alert_dialog::*;
use hydroxy_leptos::button::ButtonVariant;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (deleted, set_deleted) = signal(false);

    view! {
        <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-5);">
            // A destructive confirmation. Cancel has focus when it opens.
            <AlertDialog
                trigger="Delete project"
                title="Delete Q3 customer research?"
                description="This removes 12 files and 3 summaries. It can't be undone."
                action_label="Delete project"
                cancel_label="Keep project"
                destructive=true
                on_action=move |()| set_deleted.set(true)
            />
            // Not destructive: the confirm button is the primary look.
            <AlertDialog
                trigger="Leave page"
                trigger_variant=ButtonVariant::Ghost
                title="Leave without sending?"
                description="Your draft is saved, but it hasn't gone to anyone yet."
                action_label="Leave page"
            />
            <span>{move || if deleted.get() { "Project deleted." } else { "Nothing deleted yet." }}</span>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "alert-dialog",
    name: "Alert dialog",
    group: "Overlays",
    summary: "A modal that interrupts to confirm something. Escape and backdrop presses are ignored, so it closes only through its two buttons, and the safe one has focus.",
    demo,
    leptos: include_str!("alert_dialog.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/alert_dialog.rs"),
    parts: &[
        &["AlertDialog", "-", "Dialog with alert on. Renders the trigger, title, description and the two buttons"],
        &["", "trigger", "button, aria-haspopup=dialog (a Dialog trigger)"],
        &["", "content", "native <dialog> opened with showModal(), role=alertdialog"],
        &["", "title / description", "h2 labels the dialog, p describes it"],
        &["", "close", "Cancel: first in the DOM, so it takes focus on open"],
        &["", "action", "The confirm button"],
    ],
    keys: &[
        &["Escape", "Ignored: an alert dialog needs an answer"],
        &["Tab / Shift+Tab", "Moves focus between the two buttons; the page behind is inert"],
        &["Enter / Space", "Activates the focused button"],
    ],
    props: &[
        &["title", "String", "The question, as a heading"],
        &["description", "String", "What confirming does"],
        &["trigger", "Option<String>", "Label of the opening button. Leave out to open from code"],
        &["trigger_variant", "ButtonVariant", "Look of the trigger. Default secondary"],
        &["cancel_label", "String", "Default Cancel"],
        &["action_label", "String", "Default Continue"],
        &["destructive", "bool", "Danger look on the confirm button"],
        &["on_action", "Callback<()>", "Called when confirm is pressed"],
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled"],
        &["on_open_change", "Callback<bool>", "Called when the dialog opens or closes"],
        &["children", "Children", "Optional extra content between the description and the buttons"],
    ],
    events: &[],
};
