use dioxus::prelude::*;
use hydroxy_dioxus::button::{Button, ButtonVariant};
use hydroxy_dioxus::toast::{ToastOptions, Toaster, Tone, use_toasts};

#[component]
pub fn ToastDemo() -> Element {
    let mut retried = use_signal(|| 0);

    rsx! {
        // One Toaster near the root. Toasts stack bottom-right; hover pauses them.
        Toaster { on_action: move |_id| retried += 1,
            ToastButtons {}
            p { style: "margin: 0;", "Retries: {retried}" }
        }
    }
}

#[component]
fn ToastButtons() -> Element {
    let toasts = use_toasts();
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
            Button {
                on_click: move |_| {
                    toasts.push(
                        ToastOptions::new("Summary shared with the workspace")
                            .tone(Tone::Success)
                            .description("Everyone with access can read it."),
                    );
                },
                "Share summary"
            }
            Button {
                variant: ButtonVariant::Secondary,
                on_click: move |_| {
                    // Stays until dismissed, with a Retry action.
                    toasts.push(
                        ToastOptions::new("Couldn't read tickets-sept.csv")
                            .tone(Tone::Danger)
                            .description("410 rows have no ticket ID.")
                            .duration_ms(None)
                            .action("Retry"),
                    );
                },
                "Import tickets"
            }
            Button {
                variant: ButtonVariant::Ghost,
                on_click: move |_| {
                    toasts.push(ToastOptions::new("Drafts save every minute").duration_ms(Some(3000)));
                },
                "Show a note"
            }
        }
    }
}
