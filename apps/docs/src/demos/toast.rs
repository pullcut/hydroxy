use hydroxy_leptos::button::{Button, ButtonVariant};
use hydroxy_leptos::toast::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (retried, set_retried) = signal(0);

    view! {
        // One Toaster near the root. Toasts stack bottom-right; hover pauses them.
        <Toaster on_action=move |_id| set_retried.update(|n| *n += 1)>
            <ToastButtons />
            <p style="margin: 0;">"Retries: " {retried}</p>
        </Toaster>
    }
    .into_any()
}

#[component]
fn ToastButtons() -> impl IntoView {
    let toasts = use_toasts();
    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
            <Button on_click=move |_| {
                toasts.push(
                    ToastOptions::new("Summary shared with the workspace")
                        .tone(Tone::Success)
                        .description("Everyone with access can read it."),
                );
            }>"Share summary"</Button>
            <Button
                variant=ButtonVariant::Secondary
                on_click=move |_| {
                    // Stays until dismissed, with a Retry action.
                    toasts.push(
                        ToastOptions::new("Couldn't read tickets-sept.csv")
                            .tone(Tone::Danger)
                            .description("410 rows have no ticket ID.")
                            .duration_ms(None)
                            .action("Retry"),
                    );
                }
            >
                "Import tickets"
            </Button>
            <Button
                variant=ButtonVariant::Ghost
                on_click=move |_| {
                    toasts.push(ToastOptions::new("Drafts save every minute").duration_ms(Some(3000)));
                }
            >
                "Show a note"
            </Button>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "toast",
    name: "Toast",
    group: "Feedback",
    summary: "A brief notification in the corner. It closes on its own unless it needs a decision; hovering or focusing it pauses the clock.",
    demo,
    leptos: include_str!("toast.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/toast.rs"),
    parts: &[
        &[
            "Toaster",
            "region",
            "section: popover=manual, aria-live=polite, aria-label=Notifications. Provides use_toasts",
        ],
        &["(toast)", "toast", "data-tone, data-state=open|leaving. role=alert for danger"],
        &["(toast)", "icon, body, title, description", "Leading tone icon, then the text"],
        &["(toast)", "action, close", "Optional action button; Dismiss icon button"],
        &["(toast)", "timer", "Thin bar whose CSS animation is the lifetime"],
    ],
    keys: &[&["Tab", "Reaches the action and Dismiss buttons. Focus inside a toast pauses its clock"]],
    props: &[
        &["max_visible", "usize", "How many show at once. Default 3; the rest wait"],
        &["on_action", "Callback<u64>", "Called with the toast's id when its action is clicked"],
        &["use_toasts().push", "ToastOptions -> u64", "Shows a toast, returns its id"],
        &["use_toasts().dismiss", "u64", "Starts a toast's exit"],
        &[
            "ToastOptions",
            "title, description, tone, duration_ms, action",
            "Built with ToastOptions::new(title) and setters. duration_ms None stays until dismissed; default 5000",
        ],
    ],
    events: &[],
};
