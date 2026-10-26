use leptos::prelude::*;
use oxide_leptos::button::ButtonVariant;
use oxide_leptos::dialog::*;

pub fn demo() -> AnyView {
    let (confirm_open, set_confirm_open) = signal(false);

    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
            // Uncontrolled. Escape, the close icon or a press outside the panel closes it.
            <Dialog>
                <DialogTrigger variant=ButtonVariant::Primary>"Share summary"</DialogTrigger>
                <DialogContent>
                    <DialogCloseIcon />
                    <DialogTitle>"Share the Q3 summary"</DialogTitle>
                    <DialogDescription>
                        "Everyone in the workspace can read it, with sources attached. You can stop sharing at any time."
                    </DialogDescription>
                    <DialogActions>
                        <DialogClose variant=ButtonVariant::Secondary>"Cancel"</DialogClose>
                        <DialogClose variant=ButtonVariant::Accent>"Share with workspace"</DialogClose>
                    </DialogActions>
                </DialogContent>
            </Dialog>

            // Controlled alert: only its buttons close it.
            <Dialog alert=true open=confirm_open on_open_change=move |open| set_confirm_open.set(open)>
                <DialogTrigger variant=ButtonVariant::Secondary>"Delete project"</DialogTrigger>
                <DialogContent>
                    <DialogTitle>"Delete Q3 customer research?"</DialogTitle>
                    <DialogDescription>"This removes 12 files and 3 summaries. It can't be undone."</DialogDescription>
                    <DialogActions>
                        <DialogClose variant=ButtonVariant::Secondary>"Keep project"</DialogClose>
                        <DialogClose variant=ButtonVariant::Danger>"Delete project"</DialogClose>
                    </DialogActions>
                </DialogContent>
            </Dialog>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "dialog",
    name: "Dialog",
    group: "Overlays",
    summary: "A modal for confirmations and focused tasks. It traps focus, makes the page behind inert and returns focus to the trigger on close.",
    demo,
    leptos: include_str!("dialog.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/dialog.rs"),
    parts: &[
        &["Dialog", "-", "Provides the machine. Renders no element"],
        &["DialogTrigger", "trigger", "button, aria-haspopup=dialog, aria-expanded, aria-controls"],
        &["DialogContent", "content", "native <dialog> opened with showModal(), role=dialog|alertdialog, data-overlay"],
        &["", "panel", "The visible card inside the dialog"],
        &["DialogTitle", "title", "h2, labels the dialog"],
        &["DialogDescription", "description", "p, describes the dialog"],
        &["DialogActions", "actions", "Right-aligned button row"],
        &["DialogClose", "close", "button that closes, even in an alert"],
        &["DialogCloseIcon", "close-icon", "Icon-only close button, aria-label=Close"],
    ],
    keys: &[
        &["Escape", "Closes the dialog, unless it's an alert"],
        &["Tab / Shift+Tab", "Moves focus within the dialog; the page behind is inert"],
        &["Enter / Space", "Activates the focused button"],
    ],
    props: &[
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled"],
        &["alert", "bool", "role=alertdialog. Escape and backdrop presses don't close it"],
        &["on_open_change", "Callback<bool>", "Called when the dialog opens or closes"],
        &[
            "variant",
            "Option<ButtonVariant>",
            "On DialogTrigger and DialogClose: gives the button look (primary, accent, secondary, ghost or danger)",
        ],
    ],
};
